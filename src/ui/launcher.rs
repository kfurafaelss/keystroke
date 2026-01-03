use gtk4::prelude::*;
use gtk4::Box as GtkBox;
use gtk4::{
    Align, Application, ApplicationWindow, Button, CssProvider, Image, Label, Orientation, Picture,
};
use std::path::PathBuf;
use std::rc::Rc;
use tracing::debug;

const LAUNCHER_CSS: &str = r#"
.launcher-window {
    background-color: @theme_bg_color;
}

.header-title {
    font-size: 24px;
    font-weight: 800;
    color: @theme_fg_color;
    margin-top: 12px;
    margin-bottom: 8px;
}

.header-subtitle {
    font-size: 14px;
    color: @theme_unfocused_fg_color;
    margin-bottom: 32px;
}

.card-button {
    background-color: @theme_base_color;
    border-radius: 12px;
    padding: 0;
    margin: 12px;
    border: 1px solid @borders;
    transition: all 200ms ease;
}

.card-button:hover {
    border-color: @theme_selected_bg_color;
    background-color: @theme_base_color;
}

.card-content {
    padding: 24px;
}

.card-icon-bg {
    background-color: alpha(@theme_fg_color, 0.08);
    border-radius: 8px;
    padding: 8px;
    margin-bottom: 16px;
}

.card-title {
    font-size: 16px;
    font-weight: 700;
    color: @theme_fg_color;
    margin-bottom: 4px;
}

.card-desc {
    font-size: 13px;
    color: @theme_unfocused_fg_color;
    margin-bottom: 24px;
}

.card-preview {
    background-color: alpha(@theme_fg_color, 0.04);
    border-radius: 8px;
    padding: 16px;
    margin-top: 16px;
}

.footer-button {
    background: none;
    border: none;
    color: @theme_unfocused_fg_color;
    font-weight: 600;
    margin-top: 32px;
}

.footer-button:hover {
    color: @theme_fg_color;
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
        .title("Keystroke")
        .default_width(900)
        .default_height(700)
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
        .halign(Align::Center)
        .valign(Align::Center)
        .margin_top(48)
        .margin_bottom(48)
        .margin_start(48)
        .margin_end(48)
        .build();

    let logo_path = PathBuf::from("src/assets/logo.svg");
    let logo_widget = if logo_path.exists() {
        Picture::for_filename(logo_path)
    } else {
        Picture::for_filename("logo.svg")
    };
    logo_widget.set_can_shrink(true);
    logo_widget.set_content_fit(gtk4::ContentFit::ScaleDown);

    let logo_box = GtkBox::builder()
        .width_request(64)
        .height_request(64)
        .halign(Align::Center)
        .margin_bottom(16)
        .build();
    logo_box.append(&logo_widget);

    container.append(&logo_box);

    let title = Label::new(Some("Keystroke"));
    title.add_css_class("header-title");
    container.append(&title);

    let subtitle = Label::new(Some(
        "Visualize your keystrokes and shortcuts in real-time.",
    ));
    subtitle.add_css_class("header-subtitle");
    container.append(&subtitle);

    let cards_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(24)
        .halign(Align::Center)
        .homogeneous(true)
        .build();

    let on_select = Rc::new(on_select);

    let key_card = create_card(
        "Keystrokes",
        "Visualize shortcuts on screen.",
        "input-keyboard-symbolic",
        "keystroke.png",
        false,
        {
            let win = window.clone();
            let cb = on_select.clone();
            move || {
                debug!("Keystroke mode selected");
                win.set_visible(false);
                cb(DisplayMode::Keystroke);
            }
        },
    );
    cards_box.append(&key_card);

    let bubble_card = create_card(
        "Bubble Text",
        "Display context as floating bubbles.",
        "text-x-generic-symbolic",
        "bubble.png",
        true,
        {
            let win = window.clone();
            let cb = on_select.clone();
            move || {
                debug!("Bubble mode selected");
                win.set_visible(false);
                cb(DisplayMode::Bubble);
            }
        },
    );
    cards_box.append(&bubble_card);

    container.append(&cards_box);

    let pref_icon = Image::from_icon_name("emblem-system-symbolic");
    pref_icon.set_pixel_size(16);

    let pref_label = Label::new(Some("Preferences"));

    let pref_btn = Button::builder().css_classes(["footer-button"]).build();

    let inner_pref_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    inner_pref_box.append(&pref_icon);
    inner_pref_box.append(&pref_label);
    pref_btn.set_child(Some(&inner_pref_box));

    container.append(&pref_btn);

    container
}

fn create_card(
    title_text: &str,
    desc_text: &str,
    icon_name: &str,
    image_filename: &str,
    scale_image: bool,
    callback: impl Fn() + 'static,
) -> Button {
    let button = Button::builder()
        .css_classes(["card-button"])
        .width_request(300)
        .build();

    let content_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["card-content"])
        .build();

    let top_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .margin_bottom(16)
        .build();

    let icon_wrapper = GtkBox::builder().css_classes(["card-icon-bg"]).build();
    let icon = Image::from_icon_name(icon_name);
    icon.set_pixel_size(24);
    icon_wrapper.append(&icon);

    let spacer = GtkBox::builder().hexpand(true).build();

    let arrow = Image::from_icon_name("go-next-symbolic");
    arrow.set_opacity(0.3);

    top_row.append(&icon_wrapper);
    top_row.append(&spacer);
    top_row.append(&arrow);
    content_box.append(&top_row);

    let title = Label::builder()
        .label(title_text)
        .css_classes(["card-title"])
        .halign(Align::Start)
        .build();
    content_box.append(&title);

    let desc = Label::builder()
        .label(desc_text)
        .css_classes(["card-desc"])
        .halign(Align::Start)
        .wrap(true)
        .build();
    content_box.append(&desc);

    let mut path = PathBuf::from("src/assets");
    path.push(image_filename);

    let image_widget = if path.exists() {
        Picture::for_filename(path)
    } else {
        Picture::for_filename(image_filename)
    };

    let preview_box = GtkBox::builder()
        .css_classes(["card-preview"])
        .height_request(220)
        .build();

    if scale_image {
        image_widget.set_content_fit(gtk4::ContentFit::Contain);
    } else {
        image_widget.set_content_fit(gtk4::ContentFit::ScaleDown);
    }

    image_widget.set_can_shrink(true);

    preview_box.append(&image_widget);
    content_box.append(&preview_box);

    button.set_child(Some(&content_box));
    button.connect_clicked(move |_| callback());

    button
}

pub fn show_launcher(window: &ApplicationWindow) {
    window.set_visible(true);
    window.present();
}

#[allow(dead_code)]
pub fn hide_launcher(window: &ApplicationWindow) {
    window.set_visible(false);
}
