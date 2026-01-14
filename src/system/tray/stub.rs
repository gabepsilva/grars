//! Stub implementation for non-macOS platforms

use std::sync::mpsc;

/// System tray handle (stub)
pub struct SystemTray {
    _receiver: mpsc::Receiver<TrayEvent>,
}

/// Events from the system tray
#[derive(Debug, Clone)]
pub enum TrayEvent {
    ShowWindow,
    HideWindow,
    ReadSelected,
    Quit,
}

impl SystemTray {
    /// Create system tray (stub - does nothing on non-macOS)
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (_sender, receiver) = mpsc::channel();
        Ok(Self {
            _receiver: receiver,
        })
    }
    
    /// Try to receive a tray event (always returns None on non-macOS)
    pub fn try_recv(&self) -> Option<TrayEvent> {
        None
    }
}
