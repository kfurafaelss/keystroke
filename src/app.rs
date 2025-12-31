use crate::config::Config;
use crate::input::{KeyEvent, KeyListener, ListenerConfig};
use crate::ui::{
    create_launcher_window, create_window, setup_drag, show_launcher, DisplayMode, KeyDisplayWidget,
};
use anyhow::Result;
use async_channel::{bounded, Receiver};
use glib::ControlFlow;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use tracing::{debug, error, info};

pub struct App {
    gtk_app: Application,

    config: Config,
}

struct RuntimeState {
    mode: Option<DisplayMode>,

    paused: bool,

    keystroke_window: Option<ApplicationWindow>,

    launcher_window: Option<ApplicationWindow>,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            mode: None,
            paused: false,
            keystroke_window: None,
            launcher_window: None,
        }
    }
}

impl App {
    pub fn new(config: Config) -> Result<Self> {
        let gtk_app = Application::builder()
            .application_id("dev.linuxmobile.keystroke")
            .build();

        Ok(Self { gtk_app, config })
    }

    pub fn run(self) -> Result<i32> {
        let config = self.config.clone();

        self.gtk_app.connect_activate(move |app| {
            if let Err(e) = activate(app, &config) {
                error!("Failed to activate application: {}", e);
            }
        });

        let exit_code = self.gtk_app.run_with_args::<&str>(&[]);

        Ok(exit_code.into())
    }
}

fn activate(app: &Application, config: &Config) -> Result<()> {
    info!("Activating keystroke application");

    let state = Rc::new(RefCell::new(RuntimeState::default()));
    let config = Rc::new(config.clone());

    let state_clone = Rc::clone(&state);
    let config_clone = Rc::clone(&config);
    let app_clone = app.clone();

    let launcher = create_launcher_window(app, move |mode| {
        debug!("Mode selected: {:?}", mode);

        state_clone.borrow_mut().mode = Some(mode);

        match mode {
            DisplayMode::Keystroke => {
                if let Err(e) =
                    start_keystroke_mode(&app_clone, &config_clone, Rc::clone(&state_clone))
                {
                    error!("Failed to start keystroke mode: {}", e);
                }
            }
            DisplayMode::Bubble => {
                info!("Bubble mode selected (using keystroke for now)");
                if let Err(e) =
                    start_keystroke_mode(&app_clone, &config_clone, Rc::clone(&state_clone))
                {
                    error!("Failed to start bubble mode: {}", e);
                }
            }
        }
    });

    state.borrow_mut().launcher_window = Some(launcher.clone());

    show_launcher(&launcher);

    Ok(())
}

fn start_keystroke_mode(
    app: &Application,
    config: &Config,
    state: Rc<RefCell<RuntimeState>>,
) -> Result<()> {
    info!("Starting keystroke mode");

    if let Some(window) = state.borrow_mut().keystroke_window.take() {
        window.close();
    }

    let window = create_window(app, config)?;

    setup_drag(&window);

    let display = Rc::new(RefCell::new(KeyDisplayWidget::new(
        config.max_keys,
        config.display_timeout_ms,
    )));

    window.set_child(Some(display.borrow().widget()));

    let (sender, receiver) = bounded::<KeyEvent>(256);

    let listener_config = ListenerConfig {
        all_keyboards: config.all_keyboards,
        ..Default::default()
    };

    let listener = KeyListener::new(sender, listener_config);

    if let Err(e) = listener.start() {
        error!("Failed to start key listener: {}", e);

        let error_label = gtk4::Label::new(Some(&format!("Error: {}", e)));
        window.set_child(Some(&error_label));
    } else {
        let state_clone = Rc::clone(&state);
        setup_event_processing(display.clone(), receiver, state_clone);

        let state_clone = Rc::clone(&state);
        setup_cleanup_timer(display.clone(), window.clone(), state_clone);
    }

    state.borrow_mut().keystroke_window = Some(window.clone());

    window.present();

    Ok(())
}

fn setup_event_processing(
    display: Rc<RefCell<KeyDisplayWidget>>,
    receiver: Receiver<KeyEvent>,
    state: Rc<RefCell<RuntimeState>>,
) {
    glib::timeout_add_local(Duration::from_millis(16), move || {
        if state.borrow().paused {
            return ControlFlow::Continue;
        }

        while let Ok(event) = receiver.try_recv() {
            let mut display = display.borrow_mut();

            match event {
                KeyEvent::Pressed(key) => {
                    display.add_key(key);
                }
                KeyEvent::Released(key) => {
                    display.remove_key(&key);
                }
                KeyEvent::AllReleased => {
                    display.clear();
                }
            }
        }

        ControlFlow::Continue
    });
}

fn setup_cleanup_timer(
    display: Rc<RefCell<KeyDisplayWidget>>,
    window: ApplicationWindow,
    state: Rc<RefCell<RuntimeState>>,
) {
    glib::timeout_add_local(Duration::from_millis(100), move || {
        if state.borrow().paused {
            return ControlFlow::Continue;
        }

        let mut display = display.borrow_mut();
        display.remove_expired();

        if !display.has_keys() {
            window.set_visible(false);
        } else {
            window.set_visible(true);
        }

        ControlFlow::Continue
    });
}

#[allow(dead_code)]
fn toggle_pause(state: &Rc<RefCell<RuntimeState>>) -> bool {
    let mut s = state.borrow_mut();
    s.paused = !s.paused;
    s.paused
}

#[allow(dead_code)]
fn show_launcher_from_state(state: &Rc<RefCell<RuntimeState>>) {
    if let Some(launcher) = &state.borrow().launcher_window {
        show_launcher(launcher);
    }
}
