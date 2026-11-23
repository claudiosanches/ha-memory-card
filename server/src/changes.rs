//! Announces writes to the collection: the sensor publisher wakes, and pages
//! open in a browser hear about them and refetch.

use std::sync::Arc;

use tokio::sync::{Notify, watch};

#[derive(Clone)]
pub struct Changes {
    sensors: Arc<Notify>,
    /// How many changes so far, and whether the server is stopping.
    pages: Arc<watch::Sender<(u64, bool)>>,
}

impl Default for Changes {
    fn default() -> Self {
        Self {
            sensors: Arc::default(),
            pages: Arc::new(watch::Sender::new((0, false))),
        }
    }
}

impl Changes {
    pub fn announce(&self) {
        self.sensors.notify_one();
        self.pages.send_modify(|(count, _)| *count += 1);
    }

    /// Waits for the next change, for the sensor publisher.
    pub async fn next_for_sensors(&self) {
        self.sensors.notified().await;
    }

    pub fn subscribe(&self) -> watch::Receiver<(u64, bool)> {
        self.pages.subscribe()
    }

    /// Ends what pages are listening to, so stopping does not wait on them.
    pub fn close(&self) {
        self.pages.send_modify(|(_, closed)| *closed = true);
    }
}
