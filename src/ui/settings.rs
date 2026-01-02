use crate::config::{Config, Position};
use gtk4::prelude::*;
use gtk4::{
    Adjustment, Application, ApplicationWindow, Box as GtkBox, Button, CssProvider, DropDown,
    Label, Notebook, Orientation, Scale, SpinButton, StringList,
};
use std::cell::RefCell;
use std::rc::Rc;
use tracing::{debug, info};

const SETTINGS_CSS: &str = r#"
.settings-window {
    background-color: @window_bg_color;
}

.settings-content {
    padding: 20px;
}

.settings-header {
    padding: 16px 20px;
    border-bottom: 1px solid alpha(@borders, 0.5);
    background-color: alpha(@headerbar_bg_color, 0.5);
}

.settings-title {
    font-size: 1.1em;
    font-weight: 600;
    color: @window_fg_color;
}

.settings-section {
    margin-bottom: 16px;
}

.settings-section-title {
    font-size: 0.85em;
    font-weight: 600;
    color: @window_fg_color;
    opacity: 0.7;
    margin-bottom: 8px;
}

.settings-row {
    padding: 8px 12px;
    background-color: alpha(@card_bg_color, 0.5);
    border-radius: 8px;
    margin: 4px 0;
}

.settings-row:first-child {
    border-radius: 8px 8px 0 0;
}

.settings-row:last-child {
    border-radius: 0 0 8px 8px;
}

.settings-row:only-child {
    border-radius: 8px;
}

.settings-label {
    font-size: 0.95em;
    color: @window_fg_color;
}

.settings-description {
    font-size: 0.8em;
    color: alpha(@window_fg_color, 0.6);
}

notebook {
    background: transparent;
}

notebook > header {
    background: alpha(@headerbar_bg_color, 0.3);
    border-bottom: 1px solid alpha(@borders, 0.5);
    padding: 4px 8px;
}

notebook > header > tabs > tab {
    padding: 8px 16px;
    border-radius: 6px;
    margin: 2px;
    background: transparent;
    color: @window_fg_color;
    font-weight: 500;
}

notebook > header > tabs > tab:checked {
    background: @accent_bg_color;
    color: @accent_fg_color;
}

notebook > header > tabs > tab:hover:not(:checked) {
    background: alpha(@window_fg_color, 0.1);
}

.settings-button-box {
    padding: 16px 20px;
    border-top: 1px solid alpha(@borders, 0.5);
}

button.suggested-action {
    background: @accent_bg_color;
    color: @accent_fg_color;
    border-radius: 8px;
    padding: 8px 24px;
    font-weight: 600;
}

button.suggested-action:hover {
    background: shade(@accent_bg_color, 1.1);
}

button {
    border-radius: 8px;
    padding: 8px 16px;
}
"#;

const POSITION_OPTIONS: [(&str, Position); 6] = [
    ("Top Left", Position::TopLeft),
    ("Top Center", Position::TopCenter),
    ("Top Right", Position::TopRight),
    ("Bottom Left", Position::BottomLeft),
    ("Bottom Center", Position::BottomCenter),
    ("Bottom Right", Position::BottomRight),
];

pub fn create_settings_window(
    app: &Application,
    config: Rc<RefCell<Config>>,
    on_save: impl Fn(Config) + 'static,
) -> ApplicationWindow {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Keystroke Settings")
        .default_width(450)
        .default_height(550)
        .resizable(true)
        .build();

    apply_settings_css(&window);
    window.add_css_class("settings-window");

    let config_ref = config.borrow();

    let main_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .build();

    let notebook = Notebook::new();
    notebook.set_vexpand(true);

    let general_page = create_general_tab(&config_ref);
    notebook.append_page(&general_page.container, Some(&Label::new(Some("General"))));

    let keystroke_page = create_keystroke_tab(&config_ref);
    notebook.append_page(
        &keystroke_page.container,
        Some(&Label::new(Some("Keystroke"))),
    );

    let bubble_page = create_bubble_tab(&config_ref);
    notebook.append_page(&bubble_page.container, Some(&Label::new(Some("Bubble"))));

    main_box.append(&notebook);

    drop(config_ref);

    let button_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .halign(gtk4::Align::End)
        .build();
    button_box.add_css_class("settings-button-box");

    let cancel_btn = Button::with_label("Cancel");
    let save_btn = Button::with_label("Save");
    save_btn.add_css_class("suggested-action");

    button_box.append(&cancel_btn);
    button_box.append(&save_btn);
    main_box.append(&button_box);

    window.set_child(Some(&main_box));

    let window_clone = window.clone();
    cancel_btn.connect_clicked(move |_| {
        window_clone.close();
    });

    let window_clone = window.clone();
    let config_clone = config.clone();
    save_btn.connect_clicked(move |_| {
        let selected_idx = general_page.position_dropdown.selected() as usize;
        let position = POSITION_OPTIONS
            .get(selected_idx)
            .map(|(_, p)| *p)
            .unwrap_or(Position::BottomCenter);

        let new_config = Config {
            position,
            margin: general_page.margin_adj.value() as i32,
            opacity: general_page.opacity_adj.value(),
            display_timeout_ms: (keystroke_page.timeout_adj.value() * 1000.0) as u64,
            max_keys: keystroke_page.max_keys_adj.value() as usize,
            keystroke_font_size: keystroke_page.font_size_adj.value(),
            bubble_timeout_ms: (bubble_page.timeout_adj.value() * 1000.0) as u64,
            bubble_font_size: bubble_page.font_size_adj.value(),
            ..config_clone.borrow().clone()
        };

        debug!("Saving settings: {:?}", new_config.position);

        *config_clone.borrow_mut() = new_config.clone();

        if let Err(e) = new_config.save() {
            tracing::warn!("Failed to save config to file: {}", e);
        } else {
            info!("Configuration saved to file");
        }

        on_save(new_config);
        window_clone.close();
    });

    window
}

struct GeneralTabWidgets {
    container: GtkBox,
    position_dropdown: DropDown,
    margin_adj: Adjustment,
    opacity_adj: Adjustment,
}

fn create_general_tab(config: &Config) -> GeneralTabWidgets {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .build();
    container.add_css_class("settings-content");

    let position_section = create_section("Position");
    let position_row = create_row();
    let position_label = create_label("Overlay Position");
    let position_list = StringList::new(&POSITION_OPTIONS.map(|(name, _)| name));
    let position_dropdown = DropDown::new(Some(position_list), gtk4::Expression::NONE);

    let current_pos_idx = POSITION_OPTIONS
        .iter()
        .position(|(_, p)| *p == config.position)
        .unwrap_or(4) as u32;
    position_dropdown.set_selected(current_pos_idx);
    position_dropdown.set_hexpand(true);
    position_dropdown.set_halign(gtk4::Align::End);

    position_row.append(&position_label);
    position_row.append(&position_dropdown);
    position_section.append(&position_row);
    container.append(&position_section);

    let margin_section = create_section("Window Margin");
    let margin_row = create_row();
    let margin_label = create_label("Margin (pixels)");
    let margin_adj = Adjustment::new(config.margin as f64, 0.0, 200.0, 5.0, 20.0, 0.0);
    let margin_spin = SpinButton::new(Some(&margin_adj), 1.0, 0);
    margin_spin.set_hexpand(true);
    margin_spin.set_halign(gtk4::Align::End);

    margin_row.append(&margin_label);
    margin_row.append(&margin_spin);
    margin_section.append(&margin_row);
    container.append(&margin_section);

    let opacity_section = create_section("Appearance");
    let opacity_row = create_row();
    let opacity_label_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .halign(gtk4::Align::Start)
        .build();
    let opacity_label = create_label("Opacity");
    let opacity_desc = Label::new(Some("Window transparency"));
    opacity_desc.add_css_class("settings-description");
    opacity_label_box.append(&opacity_label);
    opacity_label_box.append(&opacity_desc);

    let opacity_adj = Adjustment::new(config.opacity, 0.1, 1.0, 0.1, 0.2, 0.0);
    let opacity_scale = Scale::new(Orientation::Horizontal, Some(&opacity_adj));
    opacity_scale.set_digits(1);
    opacity_scale.set_hexpand(true);
    opacity_scale.set_draw_value(true);
    opacity_scale.set_size_request(150, -1);

    opacity_row.append(&opacity_label_box);
    opacity_row.append(&opacity_scale);
    opacity_section.append(&opacity_row);
    container.append(&opacity_section);

    GeneralTabWidgets {
        container,
        position_dropdown,
        margin_adj,
        opacity_adj,
    }
}

struct KeystrokeTabWidgets {
    container: GtkBox,
    timeout_adj: Adjustment,
    max_keys_adj: Adjustment,
    font_size_adj: Adjustment,
}

fn create_keystroke_tab(config: &Config) -> KeystrokeTabWidgets {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .build();
    container.add_css_class("settings-content");

    let timeout_section = create_section("Display Timeout");
    let timeout_row = create_row();
    let timeout_label_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .halign(gtk4::Align::Start)
        .build();
    let timeout_label = create_label("Timeout (seconds)");
    let timeout_desc = Label::new(Some("How long keys stay visible"));
    timeout_desc.add_css_class("settings-description");
    timeout_label_box.append(&timeout_label);
    timeout_label_box.append(&timeout_desc);

    let timeout_adj = Adjustment::new(
        config.display_timeout_ms as f64 / 1000.0,
        0.5,
        10.0,
        0.5,
        1.0,
        0.0,
    );
    let timeout_scale = Scale::new(Orientation::Horizontal, Some(&timeout_adj));
    timeout_scale.set_digits(1);
    timeout_scale.set_hexpand(true);
    timeout_scale.set_draw_value(true);
    timeout_scale.set_size_request(150, -1);

    timeout_row.append(&timeout_label_box);
    timeout_row.append(&timeout_scale);
    timeout_section.append(&timeout_row);
    container.append(&timeout_section);

    let max_keys_section = create_section("Key Display");
    let max_keys_row = create_row();
    let max_keys_label_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .halign(gtk4::Align::Start)
        .build();
    let max_keys_label = create_label("Maximum Keys");
    let max_keys_desc = Label::new(Some("Max keys shown at once"));
    max_keys_desc.add_css_class("settings-description");
    max_keys_label_box.append(&max_keys_label);
    max_keys_label_box.append(&max_keys_desc);

    let max_keys_adj = Adjustment::new(config.max_keys as f64, 1.0, 20.0, 1.0, 5.0, 0.0);
    let max_keys_spin = SpinButton::new(Some(&max_keys_adj), 1.0, 0);
    max_keys_spin.set_hexpand(true);
    max_keys_spin.set_halign(gtk4::Align::End);

    max_keys_row.append(&max_keys_label_box);
    max_keys_row.append(&max_keys_spin);
    max_keys_section.append(&max_keys_row);
    container.append(&max_keys_section);

    let font_section = create_section("Typography");
    let font_row = create_row();
    let font_label_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .halign(gtk4::Align::Start)
        .build();
    let font_label = create_label("Font Size");
    let font_desc = Label::new(Some("Text size multiplier (em)"));
    font_desc.add_css_class("settings-description");
    font_label_box.append(&font_label);
    font_label_box.append(&font_desc);

    let font_size_adj = Adjustment::new(config.keystroke_font_size, 0.5, 3.0, 0.1, 0.5, 0.0);
    let font_size_scale = Scale::new(Orientation::Horizontal, Some(&font_size_adj));
    font_size_scale.set_digits(1);
    font_size_scale.set_hexpand(true);
    font_size_scale.set_draw_value(true);
    font_size_scale.set_size_request(150, -1);

    font_row.append(&font_label_box);
    font_row.append(&font_size_scale);
    font_section.append(&font_row);
    container.append(&font_section);

    KeystrokeTabWidgets {
        container,
        timeout_adj,
        max_keys_adj,
        font_size_adj,
    }
}

struct BubbleTabWidgets {
    container: GtkBox,
    timeout_adj: Adjustment,
    font_size_adj: Adjustment,
}

fn create_bubble_tab(config: &Config) -> BubbleTabWidgets {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .build();
    container.add_css_class("settings-content");

    let timeout_section = create_section("Display Timeout");
    let timeout_row = create_row();
    let timeout_label_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .halign(gtk4::Align::Start)
        .build();
    let timeout_label = create_label("Timeout (seconds)");
    let timeout_desc = Label::new(Some("How long bubbles stay visible"));
    timeout_desc.add_css_class("settings-description");
    timeout_label_box.append(&timeout_label);
    timeout_label_box.append(&timeout_desc);

    let timeout_adj = Adjustment::new(
        config.bubble_timeout_ms as f64 / 1000.0,
        1.0,
        60.0,
        1.0,
        5.0,
        0.0,
    );
    let timeout_scale = Scale::new(Orientation::Horizontal, Some(&timeout_adj));
    timeout_scale.set_digits(0);
    timeout_scale.set_hexpand(true);
    timeout_scale.set_draw_value(true);
    timeout_scale.set_size_request(150, -1);

    timeout_row.append(&timeout_label_box);
    timeout_row.append(&timeout_scale);
    timeout_section.append(&timeout_row);
    container.append(&timeout_section);

    let font_section = create_section("Typography");
    let font_row = create_row();
    let font_label_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .halign(gtk4::Align::Start)
        .build();
    let font_label = create_label("Font Size");
    let font_desc = Label::new(Some("Text size multiplier (em)"));
    font_desc.add_css_class("settings-description");
    font_label_box.append(&font_label);
    font_label_box.append(&font_desc);

    let font_size_adj = Adjustment::new(config.bubble_font_size, 0.5, 3.0, 0.1, 0.5, 0.0);
    let font_size_scale = Scale::new(Orientation::Horizontal, Some(&font_size_adj));
    font_size_scale.set_digits(1);
    font_size_scale.set_hexpand(true);
    font_size_scale.set_draw_value(true);
    font_size_scale.set_size_request(150, -1);

    font_row.append(&font_label_box);
    font_row.append(&font_size_scale);
    font_section.append(&font_row);
    container.append(&font_section);

    BubbleTabWidgets {
        container,
        timeout_adj,
        font_size_adj,
    }
}

fn create_section(title: &str) -> GtkBox {
    let section = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .build();
    section.add_css_class("settings-section");

    let title_label = Label::builder()
        .label(title)
        .halign(gtk4::Align::Start)
        .build();
    title_label.add_css_class("settings-section-title");
    section.append(&title_label);

    section
}

fn create_row() -> GtkBox {
    let row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();
    row.add_css_class("settings-row");
    row
}

fn create_label(text: &str) -> Label {
    let label = Label::builder()
        .label(text)
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();
    label.add_css_class("settings-label");
    label
}

fn apply_settings_css(window: &ApplicationWindow) {
    let provider = CssProvider::new();
    provider.load_from_string(SETTINGS_CSS);

    let display = gtk4::prelude::WidgetExt::display(window);

    gtk4::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

pub fn show_settings(window: &ApplicationWindow) {
    window.present();
}
