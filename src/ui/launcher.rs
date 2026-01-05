use gtk4::gdk::Texture;
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::prelude::*;
use gtk4::Box as GtkBox;
use gtk4::{Align, Application, ApplicationWindow, Button, CssProvider, Image, Label, Orientation};
use std::rc::Rc;
use tracing::debug;

const LOGO_SVG: &[u8] = include_bytes!("../assets/logo-symbolic.svg");

const LAUNCHER_CSS: &str = r"

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
    background: transparent;
    border: none;
    color: @theme_unfocused_fg_color;
    font-weight: 600;
    margin-top: 32px;
    padding: 8px 16px;
    border-radius: 8px;
    transition: all 200ms ease;
}

.footer-button:hover {
    color: @theme_fg_color;
    background-color: alpha(@theme_fg_color, 0.08);
}

.footer-button:active {
    background-color: alpha(@theme_fg_color, 0.12);
    transform: scale(0.96);
}

.key-cap {
    background-color: @theme_bg_color;
    color: @theme_fg_color;
    border-radius: 999px;
    padding: 12px 18px;
    font-size: 16px;
    box-shadow: 0 2px 4px alpha(black, 0.05);
}

.chat-bubble {
    background-color: @theme_bg_color;
    color: @theme_fg_color;
    border-radius: 0px 18px 18px 18px;
    padding: 12px 16px;
    font-size: 12px;
    font-weight: 500;
    box-shadow: 0 1px 2px alpha(black, 0.05);
}
";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    Keystroke,
    Bubble,
}

pub fn create_launcher_window(
    app: &Application,
    on_select: impl Fn(DisplayMode) + 'static,
    on_settings: impl Fn() + 'static,
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

    let content = create_launcher_content(&window, on_select, on_settings);
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
    on_settings: impl Fn() + 'static,
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

    let logo_box = GtkBox::builder()
        .width_request(80)
        .height_request(80)
        .halign(Align::Center)
        .margin_bottom(16)
        .build();

    let color = {
        let rgba = window.color();
        format!(
            "#{:02x}{:02x}{:02x}",
            (rgba.red() * 255.0) as u8,
            (rgba.green() * 255.0) as u8,
            (rgba.blue() * 255.0) as u8
        )
    };
    debug!("Logo color: {}", color);

    let svg_str = String::from_utf8_lossy(LOGO_SVG);
    let svg_with_color = svg_str.replace("currentColor", &color);

    let stream = gtk4::gio::MemoryInputStream::from_bytes(&gtk4::glib::Bytes::from(
        svg_with_color.as_bytes(),
    ));

    let logo_image = if let Ok(pixbuf) = Pixbuf::from_stream_at_scale(
        &stream,
        80,
        80,
        true,
        Option::<&gtk4::gio::Cancellable>::None,
    ) {
        let texture = Texture::for_pixbuf(&pixbuf);
        Image::from_paintable(Some(&texture))
    } else {
        Image::from_icon_name("application-x-executable")
    };

    logo_image.set_pixel_size(80);
    logo_image.add_css_class("logo-icon");
    logo_box.append(&logo_image);

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

    let keystroke_content = {
        let box_container = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .halign(Align::Center)
            .valign(Align::Center)
            .build();

        let keys = ["Ctrl + J", "G", "I", "T"];
        for key in keys {
            let label = Label::new(Some(key));
            label.add_css_class("key-cap");
            box_container.append(&label);
        }
        box_container
    };

    let key_card = create_card(
        "Keystrokes",
        "Visualize shortcuts on screen.",
        "preferences-desktop-keyboard-symbolic",
        keystroke_content,
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

    let bubble_content = {
        let box_container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(12)
            .halign(Align::Start)
            .valign(Align::Center)
            .build();

        let texts = [
            "3 years ago, I've started this channel to show my ricing.",
            "I had zero video skills at that time",
            "Then, it's has passed 100k subscribers",
        ];

        for text in texts {
            let label = Label::builder()
                .label(text)
                .wrap(true)
                .max_width_chars(25)
                .xalign(0.0)
                .css_classes(["chat-bubble"])
                .build();
            box_container.append(&label);
        }
        box_container
    };

    let bubble_card = create_card(
        "Bubble Text",
        "Display context as floating bubbles.",
        "chat-symbolic",
        bubble_content,
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

    let pref_btn = Button::builder()
        .css_classes(["footer-button"])
        .halign(Align::Center)
        .build();

    let inner_pref_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    inner_pref_box.append(&pref_icon);
    inner_pref_box.append(&pref_label);
    pref_btn.set_child(Some(&inner_pref_box));

    let window_clone = window.clone();
    pref_btn.connect_clicked(move |_| {
        debug!("Preferences button clicked");
        window_clone.set_visible(false);
        on_settings();
    });

    container.append(&pref_btn);

    container
}

fn create_card<W: IsA<gtk4::Widget>>(
    title_text: &str,
    desc_text: &str,
    icon_name: &str,
    preview_content: W,
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
    arrow.set_valign(Align::Center);

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

    let preview_box = GtkBox::builder()
        .css_classes(["card-preview"])
        .height_request(220)
        .build();

    preview_content.set_valign(Align::Center);
    preview_content.set_halign(Align::Center);

    preview_box.append(&preview_content);
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
