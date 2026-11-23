//! Announces writes to the collection: the sensor publisher wakes, and pages
//! open in a browser hear about them and refetch.

use std::sync::Arc;

use tokio::sync::{Notify, OwnedSemaphorePermit, Semaphore, watch};

/// Most pages listening at once; every open tab is one.
pub const LISTENERS_MAX: usize = 64;

#[derive(Clone)]
pub struct Changes {
    sensors: Arc<Notify>,
    /// How many changes so far, and whether the server is stopping.
    pages: Arc<watch::Sender<(u64, bool)>>,
    listeners: Arc<Semaphore>,
}

impl Default for Changes {
    fn default() -> Self {
        Self {
            sensors: Arc::default(),
            pages: Arc::new(watch::Sender::new((0, false))),
            listeners: Arc::new(Semaphore::new(LISTENERS_MAX)),
        }
    }
}

/// A page's place among the listeners, given back when it is dropped.
pub struct Listener {
    pub changes: watch::Receiver<(u64, bool)>,
    _place: OwnedSemaphorePermit,
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

    /// `None` while [`LISTENERS_MAX`] pages listen already.
    pub fn subscribe(&self) -> Option<Listener> {
        let place = self.listeners.clone().try_acquire_owned().ok()?;
        Some(Listener {
            changes: self.pages.subscribe(),
            _place: place,
        })
    }

    /// Ends what pages are listening to, so stopping does not wait on them.
    pub fn close(&self) {
        self.pages.send_modify(|(_, closed)| *closed = true);
    }
}
