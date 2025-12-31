use ksni::{self, menu::StandardItem, Icon, MenuItem, Tray, TrayService};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info};

#[derive(Debug, Clone)]
pub enum TrayAction {
    ShowLauncher,

    KeystrokeMode,

    BubbleMode,

    OpenSettings,

    TogglePause,

    Quit,
}

pub struct TrayState {
    pub paused: bool,
}

impl Default for TrayState {
    fn default() -> Self {
        Self { paused: false }
    }
}

struct KeystrokeTray {
    action_sender: Sender<TrayAction>,
    state: Arc<Mutex<TrayState>>,
}

impl Tray for KeystrokeTray {
    fn id(&self) -> String {
        "keystroke".to_string()
    }

    fn title(&self) -> String {
        "Keystroke".to_string()
    }

    fn icon_name(&self) -> String {
        "input-keyboard".to_string()
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        Vec::new()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        let state = self.state.lock().unwrap();
        let status = if state.paused { "Paused" } else { "Running" };
        ksni::ToolTip {
            icon_name: String::new(),
            icon_pixmap: Vec::new(),
            title: "Keystroke".to_string(),
            description: format!("Keystroke Visualizer - {}", status),
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        debug!("Tray left-clicked: showing launcher");
        if let Err(e) = self.action_sender.send(TrayAction::ShowLauncher) {
            error!("Failed to send tray action: {}", e);
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let state = self.state.lock().unwrap();
        let pause_label = if state.paused { "Resume" } else { "Pause" };
        drop(state);

        vec![
            MenuItem::Standard(StandardItem {
                label: "Keystroke Mode".to_string(),
                activate: Box::new(|tray: &mut Self| {
                    debug!("Tray: Keystroke mode selected");
                    let _ = tray.action_sender.send(TrayAction::KeystrokeMode);
                }),
                ..Default::default()
            }),
            MenuItem::Standard(StandardItem {
                label: "Bubble Mode".to_string(),
                activate: Box::new(|tray: &mut Self| {
                    debug!("Tray: Bubble mode selected");
                    let _ = tray.action_sender.send(TrayAction::BubbleMode);
                }),
                ..Default::default()
            }),
            MenuItem::Separator,
            MenuItem::Standard(StandardItem {
                label: "Settings".to_string(),
                activate: Box::new(|tray: &mut Self| {
                    debug!("Tray: Settings selected");
                    let _ = tray.action_sender.send(TrayAction::OpenSettings);
                }),
                ..Default::default()
            }),
            MenuItem::Standard(StandardItem {
                label: pause_label.to_string(),
                activate: Box::new(|tray: &mut Self| {
                    debug!("Tray: Toggle pause selected");
                    let _ = tray.action_sender.send(TrayAction::TogglePause);
                }),
                ..Default::default()
            }),
            MenuItem::Separator,
            MenuItem::Standard(StandardItem {
                label: "Quit".to_string(),
                activate: Box::new(|tray: &mut Self| {
                    debug!("Tray: Quit selected");
                    let _ = tray.action_sender.send(TrayAction::Quit);
                }),
                ..Default::default()
            }),
        ]
    }
}

pub struct TrayHandle {
    service: TrayService<KeystrokeTray>,
    #[allow(dead_code)]
    state: Arc<Mutex<TrayState>>,
}

impl TrayHandle {
    #[allow(dead_code)]
    pub fn set_paused(&self, paused: bool) {
        if let Ok(mut state) = self.state.lock() {
            state.paused = paused;
        }

        self.service.handle().update(|_| {});
    }

    #[allow(dead_code)]
    pub fn is_paused(&self) -> bool {
        self.state.lock().map(|s| s.paused).unwrap_or(false)
    }
}

pub fn start_tray() -> anyhow::Result<(mpsc::Receiver<TrayAction>, TrayHandle)> {
    let (sender, receiver) = mpsc::channel();
    let state = Arc::new(Mutex::new(TrayState::default()));

    let tray = KeystrokeTray {
        action_sender: sender,
        state: Arc::clone(&state),
    };

    let service = TrayService::new(tray);
    let handle = TrayHandle { service, state };

    handle.service.handle().update(|_| {});

    info!("System tray started");

    Ok((receiver, handle))
}

#[allow(dead_code)]
pub fn spawn_tray_service(_handle: &TrayHandle) {
    debug!("Tray service is running");
}
