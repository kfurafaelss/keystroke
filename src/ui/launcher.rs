use gtk4::prelude::*;
use gtk4::Box as GtkBox;
use gtk4::{Application, ApplicationWindow, Button, CssProvider, Label, Orientation};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::Cell;
use std::rc::Rc;
use tracing::debug;

const LAUNCHER_CSS: &str = r#"
.launcher-window {
    background-color: @window_bg_color;
    border-radius: 16px;
    padding: 16px 24px;
    border: 1px solid @borders;
}

.launcher-title {
    font-size: 0.9em;
    font-weight: 600;
    color: @window_fg_color;
    margin-bottom: 12px;
    opacity: 0.7;
}

.launcher-container {
    padding: 8px;
}

.launcher-button {
    background-color: @card_bg_color;
    color: @card_fg_color;
    border-radius: 12px;
    padding: 16px 32px;
    margin: 6px;
    font-weight: bold;
    font-size: 1.1em;
    border: 2px solid transparent;
    min-width: 120px;
    transition: all 200ms ease;
}

.launcher-button:hover {
    background-color: @accent_bg_color;
    color: @accent_fg_color;
    border-color: @accent_bg_color;
}

.launcher-button:active {
    opacity: 0.8;
}
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    Keystroke,
    Bubble,
}

#[derive(Debug)]
struct DragState {
    start_x: Cell<i32>,
    start_y: Cell<i32>,
}

impl Default for DragState {
    fn default() -> Self {
        Self {
            start_x: Cell::new(100),
            start_y: Cell::new(100),
        }
    }
}

pub fn create_launcher_window(
    app: &Application,
    on_select: impl Fn(DisplayMode) + 'static,
) -> ApplicationWindow {
    let window = ApplicationWindow::builder()
        .application(app)
        .decorated(false)
        .resizable(false)
        .build();

    window.init_layer_shell();

    window.set_layer(Layer::Overlay);

    window.set_namespace("keystroke-launcher");

    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::None);

    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Bottom, false);
    window.set_anchor(Edge::Right, false);

    window.set_margin(Edge::Top, 200);
    window.set_margin(Edge::Left, 400);

    window.set_exclusive_zone(0);

    apply_launcher_css(&window);

    window.add_css_class("launcher-window");

    let content = create_launcher_content(&window, on_select);
    window.set_child(Some(&content));

    setup_launcher_drag(&window);

    window
}

fn apply_launcher_css(window: &ApplicationWindow) {
    let provider = CssProvider::new();
    provider.load_from_string(LAUNCHER_CSS);

    let display = gtk4::prelude::WidgetExt::display(window);

    gtk4::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn create_launcher_content(
    window: &ApplicationWindow,
    on_select: impl Fn(DisplayMode) + 'static,
) -> GtkBox {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .build();

    container.add_css_class("launcher-container");

    let title = Label::new(Some("Select Mode"));
    title.add_css_class("launcher-title");
    container.append(&title);

    let button_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .halign(gtk4::Align::Center)
        .build();

    let on_select = Rc::new(on_select);

    let keystroke_btn = Button::with_label("Keystroke");
    keystroke_btn.add_css_class("launcher-button");
    let win = window.clone();
    let callback = Rc::clone(&on_select);
    keystroke_btn.connect_clicked(move |_| {
        debug!("Keystroke mode selected");
        win.set_visible(false);
        callback(DisplayMode::Keystroke);
    });
    button_box.append(&keystroke_btn);

    let bubble_btn = Button::with_label("Bubble");
    bubble_btn.add_css_class("launcher-button");
    let win = window.clone();
    let callback = Rc::clone(&on_select);
    bubble_btn.connect_clicked(move |_| {
        debug!("Bubble mode selected");
        win.set_visible(false);
        callback(DisplayMode::Bubble);
    });
    button_box.append(&bubble_btn);

    container.append(&button_box);

    container
}

fn setup_launcher_drag(window: &ApplicationWindow) {
    let drag_state = Rc::new(DragState::default());

    let gesture = gtk4::GestureDrag::new();
    gesture.set_button(1);

    let state = Rc::clone(&drag_state);
    let win = window.clone();
    gesture.connect_drag_begin(move |_, _, _| {
        let current_x = win.margin(Edge::Left);
        let current_y = win.margin(Edge::Top);

        state.start_x.set(current_x);
        state.start_y.set(current_y);

        debug!("Launcher drag started at ({}, {})", current_x, current_y);
    });

    let state = Rc::clone(&drag_state);
    let win = window.clone();
    gesture.connect_drag_update(move |_, offset_x, offset_y| {
        let start_x = state.start_x.get();
        let start_y = state.start_y.get();

        let new_x = (start_x + offset_x as i32).max(0);
        let new_y = (start_y + offset_y as i32).max(0);

        win.set_margin(Edge::Left, new_x);
        win.set_margin(Edge::Top, new_y);
    });

    window.add_controller(gesture);
}

pub fn show_launcher(window: &ApplicationWindow) {
    window.set_visible(true);
    window.present();
}

#[allow(dead_code)]
pub fn hide_launcher(window: &ApplicationWindow) {
    window.set_visible(false);
}
