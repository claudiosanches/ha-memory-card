use std::net::SocketAddr;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use clap::{Parser, Subcommand};
use memory_card::config::Config;
use memory_card::sensors::Sensors;
use memory_card::{AppState, db, logging, shutdown};
use tokio::sync::Notify;

/// In-flight requests get this long after SIGTERM. Together with the
/// checkpoint it must fit s6's default 3 s grace time.
const DRAIN_TIMEOUT: Duration = Duration::from_millis(1500);

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Run the web server (default)
    Serve,
    /// Print the OpenAPI document
    Openapi,
}

#[tokio::main]
async fn main() -> ExitCode {
    match Cli::parse().command.unwrap_or(Command::Serve) {
        Command::Openapi => match memory_card::openapi().to_pretty_json() {
            Ok(json) => {
                println!("{json}");
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("cannot serialize OpenAPI document: {err}");
                ExitCode::FAILURE
            }
        },
        Command::Serve => match serve().await {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => {
                tracing::error!(error = %err, "fatal");
                eprintln!("memory-card: {err}");
                ExitCode::FAILURE
            }
        },
    }
}

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    logging::init(config.log_level);
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "starting");
    if config.igdb.is_none() {
        tracing::info!("IGDB credentials not set, metadata lookup disabled");
    }

    reset_dir(&config.tmp_dir)?;
    std::fs::create_dir_all(config.covers_dir())?;

    let pool = db::open(
        &config.db_path(),
        &config.pre_migration_dir(),
        &db::MIGRATOR,
    )
    .await?;
    let listen = config.listen;
    let state = AppState::new(pool.clone(), Arc::new(config));
    let sensors = Sensors::start(state.clone());
    let app = memory_card::app(state);

    let listener = tokio::net::TcpListener::bind(listen).await?;
    tracing::info!(%listen, "listening");

    let stop = Arc::new(Notify::new());
    let server = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown({
        let stop = stop.clone();
        async move { stop.notified().await }
    });
    let mut server = std::pin::pin!(server.into_future());

    tokio::select! {
        result = &mut server => result?,
        () = shutdown::signal() => {
            tracing::info!("shutting down");
            stop.notify_one();
            if tokio::time::timeout(DRAIN_TIMEOUT, &mut server).await.is_err() {
                tracing::warn!("in-flight requests did not finish in time");
            }
        }
    }

    sensors.stop().await;
    db::close(&pool).await;
    tracing::info!("stopped");
    Ok(())
}

/// Temp files never need to survive a restart.
fn reset_dir(path: &std::path::Path) -> std::io::Result<()> {
    match std::fs::remove_dir_all(path) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => return Err(err),
    }
    std::fs::create_dir_all(path)
}
