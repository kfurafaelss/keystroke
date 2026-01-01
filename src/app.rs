use crate::config::Config;
use crate::input::{KeyEvent, KeyListener, ListenerConfig};
use crate::tray::{TrayAction, TrayHandle};
use crate::ui::{
    create_bubble_window, create_launcher_window, create_settings_window, create_window,
    setup_drag, show_launcher, show_settings, BubbleDisplayWidget, DisplayMode, KeyDisplayWidget,
};
use anyhow::Result;
use async_channel::{bounded, Receiver};
use gtk4::glib;
use gtk4::glib::ControlFlow;
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

#[derive(Default)]
struct RuntimeState {
    mode: Option<DisplayMode>,

    paused: bool,

    keystroke_window: Option<ApplicationWindow>,

    bubble_window: Option<ApplicationWindow>,

    launcher_window: Option<ApplicationWindow>,

    settings_window: Option<ApplicationWindow>,
}

impl App {
    pub fn new(config: Config) -> Result<Self> {
        let gtk_app = Application::builder()
            .application_id("dev.linuxmobile.keystroke")
            .build();

        Ok(Self { gtk_app, config })
    }

    pub fn run_with_tray(
        self,
        tray_rx: Receiver<TrayAction>,
        tray_handle: TrayHandle,
    ) -> Result<i32> {
        let config = self.config.clone();

        self.gtk_app.connect_activate(move |app| {
            if let Err(e) = activate(app, &config, tray_rx.clone(), tray_handle.clone()) {
                error!("Failed to activate application: {}", e);
            }
        });

        let exit_code = self.gtk_app.run_with_args::<&str>(&[]);

        Ok(exit_code.into())
    }

    pub fn run(self) -> Result<i32> {
        let config = self.config.clone();

        self.gtk_app.connect_activate(move |app| {
            if let Err(e) = activate_without_tray(app, &config) {
                error!("Failed to activate application: {}", e);
            }
        });

        let exit_code = self.gtk_app.run_with_args::<&str>(&[]);

        Ok(exit_code.into())
    }
}

fn activate_without_tray(app: &Application, config: &Config) -> Result<()> {
    info!("Activating keystroke application (no tray)");

    let state = Rc::new(RefCell::new(RuntimeState::default()));
    let config = Rc::new(RefCell::new(config.clone()));

    setup_launcher_and_modes(app, &state, &config);

    Ok(())
}

fn activate(
    app: &Application,
    config: &Config,
    tray_rx: Receiver<TrayAction>,
    tray_handle: TrayHandle,
) -> Result<()> {
    info!("Activating keystroke application");

    let state = Rc::new(RefCell::new(RuntimeState::default()));
    let config = Rc::new(RefCell::new(config.clone()));

    setup_launcher_and_modes(app, &state, &config);

    setup_tray_handling(
        Rc::clone(&state),
        Rc::clone(&config),
        app.clone(),
        tray_rx,
        tray_handle,
    );

    Ok(())
}

fn setup_launcher_and_modes(
    app: &Application,
    state: &Rc<RefCell<RuntimeState>>,
    config: &Rc<RefCell<Config>>,
) {
    let state_clone = Rc::clone(state);
    let config_clone = Rc::clone(config);
    let app_clone = app.clone();

    let launcher = create_launcher_window(app, move |mode| {
        debug!("Mode selected: {:?}", mode);
        switch_mode(&app_clone, &state_clone, &config_clone, mode);
    });

    state.borrow_mut().launcher_window = Some(launcher.clone());

    show_launcher(&launcher);
}

fn switch_mode(
    app: &Application,
    state: &Rc<RefCell<RuntimeState>>,
    config: &Rc<RefCell<Config>>,
    mode: DisplayMode,
) {
    close_mode_windows(state);

    state.borrow_mut().mode = Some(mode);

    match mode {
        DisplayMode::Keystroke => {
            if let Err(e) = start_keystroke_mode(app, &config.borrow(), Rc::clone(state)) {
                error!("Failed to start keystroke mode: {}", e);
            }
        }
        DisplayMode::Bubble => {
            if let Err(e) = start_bubble_mode(app, &config.borrow(), Rc::clone(state)) {
                error!("Failed to start bubble mode: {}", e);
            }
        }
    }
}

fn close_mode_windows(state: &Rc<RefCell<RuntimeState>>) {
    let mut s = state.borrow_mut();
    if let Some(window) = s.keystroke_window.take() {
        window.close();
    }
    if let Some(window) = s.bubble_window.take() {
        window.close();
    }
}

fn setup_tray_handling(
    state: Rc<RefCell<RuntimeState>>,
    config: Rc<RefCell<Config>>,
    app: Application,
    tray_rx: Receiver<TrayAction>,
    tray_handle: TrayHandle,
) {
    let tray_handle = Rc::new(tray_handle);

    glib::timeout_add_local(Duration::from_millis(50), move || {
        while let Ok(action) = tray_rx.try_recv() {
            handle_tray_action(&action, &state, &config, &app, &tray_handle);
        }
        ControlFlow::Continue
    });
}

fn handle_tray_action(
    action: &TrayAction,
    state: &Rc<RefCell<RuntimeState>>,
    config: &Rc<RefCell<Config>>,
    app: &Application,
    tray_handle: &Rc<TrayHandle>,
) {
    match action {
        TrayAction::ShowLauncher => {
            debug!("Handling ShowLauncher action");
            if let Some(launcher) = &state.borrow().launcher_window {
                show_launcher(launcher);
            }
        }
        TrayAction::KeystrokeMode => {
            debug!("Handling KeystrokeMode action");
            switch_mode(app, state, config, DisplayMode::Keystroke);
        }
        TrayAction::BubbleMode => {
            debug!("Handling BubbleMode action");
            switch_mode(app, state, config, DisplayMode::Bubble);
        }
        TrayAction::OpenSettings => {
            debug!("Handling OpenSettings action");
            open_settings(app, state, config);
        }
        TrayAction::TogglePause => {
            debug!("Handling TogglePause action");
            let paused = toggle_pause(state);
            tray_handle.set_paused(paused);
            info!(
                "Keystroke capture {}",
                if paused { "paused" } else { "resumed" }
            );
        }
        TrayAction::Quit => {
            debug!("Handling Quit action");
            app.quit();
        }
    }
}

fn open_settings(
    app: &Application,
    state: &Rc<RefCell<RuntimeState>>,
    config: &Rc<RefCell<Config>>,
) {
    if let Some(ref settings_window) = state.borrow().settings_window {
        show_settings(settings_window);
        return;
    }

    let state_clone = Rc::clone(state);
    let config_clone = Rc::clone(config);
    let app_clone = app.clone();

    let settings_window = create_settings_window(app, Rc::clone(config), move |_new_config| {
        info!("Settings saved, applying changes");

        let current_mode = state_clone.borrow().mode;
        if let Some(mode) = current_mode {
            close_mode_windows(&state_clone);
            switch_mode(&app_clone, &state_clone, &config_clone, mode);
        }
    });

    state.borrow_mut().settings_window = Some(settings_window.clone());

    let state_clone = Rc::clone(state);
    settings_window.connect_close_request(move |_| {
        state_clone.borrow_mut().settings_window = None;
        glib::Propagation::Proceed
    });

    show_settings(&settings_window);
}

fn start_keystroke_mode(
    app: &Application,
    config: &Config,
    state: Rc<RefCell<RuntimeState>>,
) -> Result<()> {
    info!("Starting keystroke mode");

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
        setup_keystroke_event_processing(display.clone(), receiver, state_clone);

        let state_clone = Rc::clone(&state);
        setup_keystroke_cleanup_timer(display.clone(), window.clone(), state_clone);
    }

    state.borrow_mut().keystroke_window = Some(window.clone());

    window.present();

    Ok(())
}

fn start_bubble_mode(
    app: &Application,
    config: &Config,
    state: Rc<RefCell<RuntimeState>>,
) -> Result<()> {
    info!("Starting bubble mode");

    let window = create_bubble_window(app, config)?;

    setup_drag(&window);

    let display = Rc::new(RefCell::new(BubbleDisplayWidget::new(
        config.bubble_timeout_ms,
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
        setup_bubble_event_processing(display.clone(), receiver, state_clone);

        let state_clone = Rc::clone(&state);
        setup_bubble_cleanup_timer(display.clone(), window.clone(), state_clone);
    }

    state.borrow_mut().bubble_window = Some(window.clone());

    window.present();

    Ok(())
}

fn setup_keystroke_event_processing(
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

fn setup_bubble_event_processing(
    display: Rc<RefCell<BubbleDisplayWidget>>,
    receiver: Receiver<KeyEvent>,
    state: Rc<RefCell<RuntimeState>>,
) {
    glib::timeout_add_local(Duration::from_millis(16), move || {
        if state.borrow().paused {
            return ControlFlow::Continue;
        }

        while let Ok(event) = receiver.try_recv() {
            if let KeyEvent::Pressed(key) = event {
                let mut display = display.borrow_mut();
                display.process_key(key);
            }
        }

        ControlFlow::Continue
    });
}

fn setup_keystroke_cleanup_timer(
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

fn setup_bubble_cleanup_timer(
    display: Rc<RefCell<BubbleDisplayWidget>>,
    window: ApplicationWindow,
    state: Rc<RefCell<RuntimeState>>,
) {
    glib::timeout_add_local(Duration::from_millis(100), move || {
        if state.borrow().paused {
            return ControlFlow::Continue;
        }

        let mut display = display.borrow_mut();
        display.remove_expired();

        if display.has_content() {
            window.set_visible(true);
        } else {
            window.set_visible(false);
        }

        ControlFlow::Continue
    });
}

fn toggle_pause(state: &Rc<RefCell<RuntimeState>>) -> bool {
    let mut s = state.borrow_mut();
    s.paused = !s.paused;
    s.paused
}
