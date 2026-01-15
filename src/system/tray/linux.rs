//! Linux system tray implementation

use std::sync::mpsc;
use std::thread;
use tray_icon::{
    menu::{Menu, MenuItem, MenuEvent, PredefinedMenuItem},
    TrayIconBuilder,
};
use tracing::{info, warn};
use crate::system::{HotkeyConfig, format_hotkey_display};

// Embedded logo asset - using PNG file for Linux (same as macOS)
const LOGO_PNG: &[u8] = include_bytes!("../../../assets/logo.png");

/// System tray handle
pub struct SystemTray {
    _tray_icon: Option<()>, // Placeholder - actual TrayIcon lives in GTK thread
    _gtk_thread: Option<thread::JoinHandle<()>>,
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
    pub fn new(hotkey_config: Option<&HotkeyConfig>) -> Result<Self, Box<dyn std::error::Error>> {
        let (sender, receiver) = mpsc::channel();
        
        // Prepare data for the GTK thread
        let read_selected_label = if let Some(config) = hotkey_config {
            let hotkey_display = format_hotkey_display(config);
            format!("Read Selected\t{}", hotkey_display)
        } else {
            "Read Selected".to_string()
        };
        
        // Load icon data before spawning thread (this doesn't require GTK)
        let icon_data = match load_tray_icon_from_logo()
            .and_then(|(rgba_data, width, height)| {
                tray_icon::Icon::from_rgba(rgba_data, width, height)
                    .map_err(|e| format!("Failed to create icon from RGBA data: {e}").into())
            }) {
            Ok(icon) => icon,
            Err(e) => {
                warn!(error = %e, "Failed to load tray icon, continuing without tray icon");
                return Ok(Self {
                    _tray_icon: None,
                    _gtk_thread: None,
                    receiver,
                });
            }
        };
        
        // For Linux, we need to run GTK in a separate thread
        // The tray-icon crate requires a GTK event loop running on the thread where the icon is created
        // Since Iced has its own event loop, we spawn a dedicated GTK thread
        let (tray_ready_tx, tray_ready_rx) = std::sync::mpsc::channel();
        let sender_for_thread = sender.clone();
        let gtk_thread = thread::spawn(move || {
            // Initialize GTK in this thread
            if let Err(e) = gtk::init() {
                warn!(error = %e, "Failed to initialize GTK for system tray, tray icon will not be available");
                let _ = tray_ready_tx.send(None);
                return;
            }
            
            // Now create the tray icon in this GTK thread
            let read_selected_item = MenuItem::new(&read_selected_label, true, None);
            let show_item = MenuItem::new("Show Window", true, None);
            let hide_item = MenuItem::new("Hide Window", true, None);
            let quit_item = MenuItem::new("Quit", true, None);
            
            let read_selected_id = read_selected_item.id();
            let show_id = show_item.id();
            let hide_id = hide_item.id();
            let quit_id = quit_item.id();
            
            let separator = PredefinedMenuItem::separator();
            let menu = Menu::new();
            if let Err(e) = menu.append(&read_selected_item) {
                warn!(error = %e, "Failed to append menu item");
                let _ = tray_ready_tx.send(None);
                return;
            }
            menu.append(&separator).ok();
            menu.append(&show_item).ok();
            menu.append(&hide_item).ok();
            menu.append(&separator).ok();
            menu.append(&quit_item).ok();
            
            // Set up menu event handler
            let sender_clone = sender_for_thread.clone();
            let show_id = show_id.clone();
            let hide_id = hide_id.clone();
            let read_selected_id = read_selected_id.clone();
            let quit_id = quit_id.clone();
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
            let tray_result = TrayIconBuilder::new()
                .with_menu(Box::new(menu))
                .with_tooltip("Insight Reader")
                .with_icon(icon_data)
                .build();
            
            match tray_result {
                Ok(_tray_icon) => {
                    info!("System tray icon created successfully");
                    let _ = tray_ready_tx.send(Some(()));
                    // Keep GTK event loop running (this blocks, but that's OK in a separate thread)
                    gtk::main();
                }
                Err(e) => {
                    warn!(error = %e, "Failed to create tray icon, continuing without tray icon");
                    let _ = tray_ready_tx.send(None);
                }
            }
        });
        
        // Wait for tray icon to be created (with timeout)
        let tray_created = tray_ready_rx
            .recv_timeout(std::time::Duration::from_secs(2))
            .ok()
            .flatten()
            .is_some();
        
        if !tray_created {
            warn!("Tray icon creation timeout or failure, continuing without tray icon");
        }
        
        // Note: We don't store the TrayIcon here because it must stay in the GTK thread
        // The GTK thread keeps it alive, and we just need to keep the thread running
        Ok(Self {
            _tray_icon: tray_created.then_some(()),
            _gtk_thread: Some(gtk_thread),
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
    // Decode PNG and convert to RGBA
    let img = image::load_from_memory(LOGO_PNG)
        .map_err(|e| format!("Failed to decode logo PNG: {e}"))?
        .to_rgba8();
    
    // Resize to 24x24 for Linux tray icons (common size)
    const TARGET_SIZE: u32 = 24;
    let rgba_data = image::imageops::resize(
        &img,
        TARGET_SIZE,
        TARGET_SIZE,
        image::imageops::FilterType::Lanczos3,
    )
    .into_raw();
    
    Ok((rgba_data, TARGET_SIZE, TARGET_SIZE))
}
