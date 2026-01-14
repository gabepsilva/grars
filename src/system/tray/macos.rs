//! macOS system tray implementation

use std::sync::mpsc;
use tray_icon::{
    menu::{Menu, MenuItem, MenuEvent},
    TrayIconBuilder, TrayIcon,
};
use tracing::info;

/// System tray handle
pub struct SystemTray {
    _tray_icon: TrayIcon,
    receiver: mpsc::Receiver<TrayEvent>,
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
    /// Create and initialize the system tray icon
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (sender, receiver) = mpsc::channel();
        
        // Create menu items
        let show_item = MenuItem::new("Show Window", true, None);
        let hide_item = MenuItem::new("Hide Window", true, None);
        let read_selected_item = MenuItem::new("Read Selected", true, None);
        let quit_item = MenuItem::new("Quit", true, None);
        
        // Store menu item IDs
        let show_item_id = show_item.id();
        let hide_item_id = hide_item.id();
        let read_selected_item_id = read_selected_item.id();
        let quit_item_id = quit_item.id();
        
        // Create menu
        let menu = Menu::new();
        menu.append(&show_item)?;
        menu.append(&hide_item)?;
        menu.append(&read_selected_item)?;
        menu.append(&quit_item)?;
        
        // Create a simple icon from bytes (16x16 black square with transparency)
        // This is a minimal template icon for macOS menu bar
        let icon_data = create_tray_icon();
        let icon = tray_icon::Icon::from_rgba(icon_data, 16, 16)
            .map_err(|e| format!("Failed to create icon: {e}"))?;
        
        // Set up menu event handler before creating the tray icon
        let sender_clone = sender.clone();
        let show_id = show_item_id.clone();
        let hide_id = hide_item_id.clone();
        let read_selected_id = read_selected_item_id.clone();
        let quit_id = quit_item_id.clone();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            let event_to_send = match event.id {
                id if id == show_id => Some(TrayEvent::ShowWindow),
                id if id == hide_id => Some(TrayEvent::HideWindow),
                id if id == read_selected_id => Some(TrayEvent::ReadSelected),
                id if id == quit_id => Some(TrayEvent::Quit),
                _ => None,
            };
            
            if let Some(evt) = event_to_send {
                let _ = sender_clone.send(evt);
            }
        }));
        
        // Create tray icon
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Insight Reader")
            .with_icon(icon)
            .build()
            .map_err(|e| format!("Failed to create tray icon: {e}"))?;
        
        info!("System tray icon created successfully");
        
        Ok(Self {
            _tray_icon: tray_icon,
            receiver,
        })
    }
    
    /// Try to receive a tray event (non-blocking)
    pub fn try_recv(&self) -> Option<TrayEvent> {
        self.receiver.try_recv().ok()
    }
}

/// Create a simple 16x16 RGBA icon for the tray (black circle)
fn create_tray_icon() -> Vec<u8> {
    let mut data = vec![0u8; 16 * 16 * 4];
    let center_x = 8.0;
    let center_y = 8.0;
    
    for y in 0..16 {
        for x in 0..16 {
            let idx = (y * 16 + x) * 4;
            let dist = ((x as f32 - center_x).powi(2) + (y as f32 - center_y).powi(2)).sqrt();
            
            data[idx + 3] = if dist < 6.0 {
                255 // Opaque fill
            } else if dist < 7.0 {
                200 // Border
            } else {
                0 // Transparent
            };
        }
    }
    
    data
}
