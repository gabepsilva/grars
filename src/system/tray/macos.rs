//! macOS system tray implementation

use std::sync::mpsc;
use tray_icon::{
    menu::{Menu, MenuItem, MenuEvent},
    TrayIconBuilder, TrayIcon,
};
use tracing::info;

// Embedded logo asset
const LOGO_PNG: &[u8] = include_bytes!("../../../assets/logo.png");

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
        
        // Load and resize the app logo for the tray icon
        // macOS menu bar icons are typically 16x16 or 22x22 (retina)
        let icon_data = load_tray_icon_from_logo()?;
        let icon = tray_icon::Icon::from_rgba(icon_data.0, icon_data.1, icon_data.2)
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

/// Load the app logo and convert it to RGBA format for the tray icon
/// Returns (rgba_data, width, height)
fn load_tray_icon_from_logo() -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error>> {
    // Decode the PNG logo
    let img = image::load_from_memory(LOGO_PNG)
        .map_err(|e| format!("Failed to decode logo PNG: {e}"))?;
    
    // Convert to RGBA format
    let rgba_img = img.to_rgba8();
    
    // Resize to 16x16 for standard display, or 22x22 for retina
    // Using 22x22 for better quality on retina displays
    let target_size = 22u32;
    let resized = image::imageops::resize(
        &rgba_img,
        target_size,
        target_size,
        image::imageops::FilterType::Lanczos3,
    );
    
    // Convert to Vec<u8> in RGBA format
    let rgba_data = resized.into_raw();
    
    Ok((rgba_data, target_size, target_size))
}
