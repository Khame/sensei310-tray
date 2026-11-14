// SPDX-FileCopyrightText: 2026 Khame
// SPDX-License-Identifier: MIT

//! Shared answer to "is the mouse on the USB bus right now?".
//!
//! The profile watcher reads sysfs every few seconds and records the result
//! here. The tray menu reads it to grey out every action that needs the device,
//! and a change wakes the tray so the open menu is rebuilt.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::sync::watch;

/// Connection state shared by the profile watcher and the tray menu.
#[derive(Clone)]
pub struct Connection {
    connected: Arc<AtomicBool>,
    changes: watch::Sender<bool>,
}

impl Connection {
    pub fn new() -> Self {
        Self {
            connected: Arc::new(AtomicBool::new(false)),
            changes: watch::channel(false).0,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    /// Record a fresh reading. Returns `true` when it differs from the last one.
    pub fn set(&self, connected: bool) -> bool {
        if self.connected.swap(connected, Ordering::Relaxed) == connected {
            return false;
        }
        self.changes.send_replace(connected);
        true
    }

    /// Wake up whenever [`set`](Self::set) reports a different state.
    pub fn subscribe(&self) -> watch::Receiver<bool> {
        self.changes.subscribe()
    }
}

impl Default for Connection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_disconnected() {
        assert!(!Connection::new().is_connected());
    }

    #[test]
    fn reports_only_actual_changes() {
        let connection = Connection::new();

        assert!(connection.set(true));
        assert!(connection.is_connected());
        assert!(!connection.set(true));
        assert!(connection.set(false));
        assert!(!connection.is_connected());
    }

    #[test]
    fn subscribers_see_every_change() {
        let connection = Connection::new();
        let mut changes = connection.subscribe();

        connection.set(true);
        assert!(changes.has_changed().unwrap());
        assert!(*changes.borrow_and_update());

        connection.set(true);
        assert!(!changes.has_changed().unwrap());

        connection.set(false);
        assert!(!*changes.borrow_and_update());
    }
}
