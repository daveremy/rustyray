//! Test utilities for synchronization and coordination in tests

use std::sync::Arc;
use tokio::sync::{oneshot, Barrier, Notify};

/// A synchronization point that can be used to coordinate test execution
pub struct TestSync {
    /// Barrier for ensuring all participants have reached a point
    pub barrier: Arc<Barrier>,
    /// Notification for signaling events
    pub notify: Arc<Notify>,
}

impl TestSync {
    /// Create a new test synchronization helper
    pub fn new(num_participants: usize) -> Self {
        TestSync {
            barrier: Arc::new(Barrier::new(num_participants)),
            notify: Arc::new(Notify::new()),
        }
    }

    /// Create a simple notification-based synchronization
    pub fn notification() -> Arc<Notify> {
        Arc::new(Notify::new())
    }
}

/// A channel-based gate for test synchronization
pub struct TestGate {
    tx: Option<oneshot::Sender<()>>,
    rx: Option<oneshot::Receiver<()>>,
}

impl TestGate {
    /// Create a new test gate
    pub fn new() -> (Self, Self) {
        let (tx, rx) = oneshot::channel();
        (
            TestGate {
                tx: Some(tx),
                rx: None,
            },
            TestGate {
                tx: None,
                rx: Some(rx),
            },
        )
    }

    /// Open the gate (sender side)
    pub fn open(mut self) {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(());
        }
    }

    /// Wait for the gate to open (receiver side)
    pub async fn wait(&mut self) {
        if let Some(rx) = self.rx.take() {
            let _ = rx.await;
        }
    }
}
