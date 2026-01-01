use gtk4::prelude::*;
use gtk4::Box as GtkBox;
use gtk4::{Application, ApplicationWindow, Button, CssProvider, Label, Orientation};
use std::rc::Rc;
use tracing::debug;

const LAUNCHER_CSS: &str = r#"
.launcher-window {
    background-color: @window_bg_color;
    border-radius: 12px;
    padding: 16px 24px;
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

pub fn create_launcher_window(
    app: &Application,
    on_select: impl Fn(DisplayMode) + 'static,
) -> ApplicationWindow {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Keystroke - Select Mode")
        .default_width(350)
        .default_height(150)
        .resizable(false)
        .build();

    apply_launcher_css(&window);

    window.add_css_class("launcher-window");

    let content = create_launcher_content(&window, on_select);
    window.set_child(Some(&content));

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
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
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

pub fn show_launcher(window: &ApplicationWindow) {
    window.set_visible(true);
    window.present();
}

#[allow(dead_code)]
pub fn hide_launcher(window: &ApplicationWindow) {
    window.set_visible(false);
}
