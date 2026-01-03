use crate::config::{Config, Position};
use gtk4::prelude::*;
use gtk4::{
    Adjustment, Application, ApplicationWindow, Box as GtkBox, Button, ColorDialog,
    ColorDialogButton, CssProvider, DropDown, Entry, Label, Orientation, Scale, SpinButton, Stack,
    StackSidebar, StringList, Switch,
};
use std::cell::RefCell;
use std::rc::Rc;
use tracing::{debug, info};

const SETTINGS_CSS: &str = r#"
.settings-window {
    background-color: @theme_bg_color;
}

.settings-sidebar {
    background-color: mix(@theme_bg_color, @theme_base_color, 0.5);
    border-right: 1px solid @borders;
    padding: 8px 0;
}

.settings-sidebar row {
    padding: 12px 20px;
    margin: 2px 8px;
    border-radius: 8px;
}

.settings-sidebar row:selected {
    background-color: alpha(@theme_selected_bg_color, 0.2);
}

.settings-content-area {
    padding: 32px 48px;
    background-color: @theme_bg_color;
}

.settings-page-header {
    margin-bottom: 32px;
}

.settings-page-icon {
    background: linear-gradient(135deg, @purple_3, @pink_3);
    border-radius: 12px;
    padding: 16px;
    min-width: 48px;
    min-height: 48px;
}

.settings-page-title {
    font-size: 24px;
    font-weight: 700;
    color: @theme_fg_color;
}

.settings-page-subtitle {
    font-size: 14px;
    color: @theme_unfocused_fg_color;
    margin-top: 4px;
}

.settings-section-title {
    font-size: 12px;
    font-weight: 700;
    color: @theme_unfocused_fg_color;
    letter-spacing: 0.5px;
    margin-bottom: 16px;
    margin-top: 32px;
    text-transform: uppercase;
}

.settings-card {
    background-color: @theme_base_color;
    border-radius: 12px;
    border: 1px solid @borders;
    padding: 4px 0;
}

.settings-row {
    padding: 16px 20px;
    min-height: 48px;
}

.settings-row:not(:last-child) {
    border-bottom: 1px solid alpha(@borders, 0.5);
}

.settings-label {
    font-weight: 500;
    color: @theme_fg_color;
    font-size: 14px;
}

.settings-sublabel {
    font-size: 12px;
    color: @theme_unfocused_fg_color;
    margin-top: 2px;
}

.suggested-action {
    background-color: @theme_selected_bg_color;
    color: @theme_selected_fg_color;
    border: none;
    border-radius: 8px;
    padding: 8px 20px;
    font-weight: 600;
}

.suggested-action:hover {
    background-color: shade(@theme_selected_bg_color, 0.9);
}

.cancel-button {
    background-color: alpha(@theme_fg_color, 0.1);
    color: @theme_fg_color;
    border: none;
    border-radius: 8px;
    padding: 8px 20px;
    font-weight: 600;
}

.cancel-button:hover {
    background-color: alpha(@theme_fg_color, 0.15);
}

.theme-button {
    background-color: alpha(@theme_fg_color, 0.08);
    border: 1px solid @borders;
    border-radius: 6px;
    padding: 8px 16px;
    font-size: 13px;
    font-weight: 500;
    color: @theme_fg_color;
    min-width: 60px;
}

.theme-button:hover {
    background-color: alpha(@theme_fg_color, 0.12);
}

.theme-button.selected {
    background-color: @theme_selected_bg_color;
    color: @theme_selected_fg_color;
    border-color: @theme_selected_bg_color;
}

.color-button-row button {
    border-radius: 50%;
    min-width: 32px;
    min-height: 32px;
    padding: 0;
    border: 2px solid transparent;
}

.color-button-row button:checked,
.color-button-row button.selected {
    border-color: @theme_selected_bg_color;
}

.position-grid {
    background-color: alpha(@theme_fg_color, 0.06);
    border-radius: 8px;
    padding: 8px;
}

.position-button {
    background-color: alpha(@theme_fg_color, 0.12);
    border: none;
    border-radius: 4px;
    min-width: 24px;
    min-height: 24px;
    margin: 2px;
}

.position-button:checked,
.position-button.selected {
    background-color: @theme_selected_bg_color;
}

.flat-entry {
    border: 1px solid @borders;
    border-radius: 6px;
    padding: 8px 12px;
    background-color: @theme_base_color;
    color: @theme_fg_color;
}

.flat-entry:focus {
    border-color: @theme_selected_bg_color;
}

scale {
    min-width: 180px;
}

scale trough {
    background-color: alpha(@theme_fg_color, 0.12);
    border-radius: 4px;
    min-height: 6px;
}

scale highlight {
    background-color: @theme_selected_bg_color;
    border-radius: 4px;
}

scale slider {
    background-color: @theme_base_color;
    border: 1px solid @borders;
    border-radius: 50%;
    min-width: 18px;
    min-height: 18px;
}

switch {
    background-color: alpha(@theme_fg_color, 0.2);
    border-radius: 14px;
}

switch:checked {
    background-color: @theme_selected_bg_color;
}

switch slider {
    background-color: @theme_base_color;
    border-radius: 50%;
}

dropdown button {
    background-color: alpha(@theme_fg_color, 0.06);
    border: 1px solid @borders;
    border-radius: 6px;
    padding: 6px 12px;
    color: @theme_fg_color;
}

dropdown button:hover {
    background-color: alpha(@theme_fg_color, 0.1);
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

const THEME_OPTIONS: [&str; 3] = ["Light", "Dark", "System"];

pub fn create_settings_window(
    app: &Application,
    config: Rc<RefCell<Config>>,
    on_save: impl Fn(Config) + 'static,
) -> ApplicationWindow {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Settings")
        .default_width(900)
        .default_height(650)
        .resizable(true)
        .build();

    apply_settings_css(&window);
    window.add_css_class("settings-window");

    let main_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .build();

    let stack = Stack::new();
    stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
    stack.set_transition_duration(200);
    stack.set_hexpand(true);
    stack.set_vexpand(true);

    let sidebar = StackSidebar::new();
    sidebar.set_stack(&stack);
    sidebar.add_css_class("settings-sidebar");
    sidebar.set_size_request(200, -1);

    let config_ref = config.borrow();

    let keystroke_page = create_keystroke_page(&config_ref);
    stack.add_titled(&keystroke_page.container, Some("keystroke"), "Keystroke");

    let bubble_page = create_bubble_page(&config_ref);
    stack.add_titled(&bubble_page.container, Some("bubble"), "Bubble");

    main_box.append(&sidebar);

    let content_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .build();

    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .vexpand(true)
        .child(&stack)
        .build();

    content_box.append(&scrolled);

    let action_bar = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .margin_top(16)
        .margin_bottom(16)
        .margin_end(24)
        .halign(gtk4::Align::End)
        .build();

    let cancel_btn = Button::with_label("Cancel");
    cancel_btn.add_css_class("cancel-button");
    let save_btn = Button::with_label("Save");
    save_btn.add_css_class("suggested-action");

    action_bar.append(&cancel_btn);
    action_bar.append(&save_btn);
    content_box.append(&action_bar);

    main_box.append(&content_box);
    window.set_child(Some(&main_box));

    let window_clone = window.clone();
    cancel_btn.connect_clicked(move |_| {
        window_clone.close();
    });

    let window_clone = window.clone();
    let config_clone = config.clone();

    save_btn.connect_clicked(move |_| {
        let theme_idx = keystroke_page.theme_dropdown.selected();
        let theme = THEME_OPTIONS
            .get(theme_idx as usize)
            .unwrap_or(&"System")
            .to_lowercase();

        let ks_pos_idx = keystroke_page.position_dropdown.selected();
        let ks_position = POSITION_OPTIONS
            .get(ks_pos_idx as usize)
            .map(|(_, p)| *p)
            .unwrap_or(Position::BottomCenter);

        let b_pos_idx = bubble_page.position_dropdown.selected();
        let b_position = POSITION_OPTIONS
            .get(b_pos_idx as usize)
            .map(|(_, p)| *p)
            .unwrap_or(Position::TopRight);

        let rgba = bubble_page.color_button.rgba();
        let color_hex = format!(
            "#{:02x}{:02x}{:02x}",
            (rgba.red() * 255.0) as u8,
            (rgba.green() * 255.0) as u8,
            (rgba.blue() * 255.0) as u8
        );

        let new_config = Config {
            keystroke_theme: theme,
            display_timeout_ms: (keystroke_page.duration_adj.value() * 1000.0) as u64,
            max_keys: keystroke_page.max_keys_adj.value() as usize,
            position: ks_position,
            keystroke_draggable: keystroke_page.draggable_switch.is_active(),
            keystroke_hotkey: keystroke_page.hotkey_entry.text().to_string(),

            bubble_color: color_hex,
            bubble_font_size: bubble_page.font_size_adj.value(),
            bubble_font_family: bubble_page.font_entry.text().to_string(),
            bubble_hotkey: bubble_page.hotkey_entry.text().to_string(),
            bubble_sound_enabled: bubble_page.sound_switch.is_active(),
            bubble_position: b_position,
            bubble_draggable: bubble_page.draggable_switch.is_active(),
            bubble_timeout_ms: (bubble_page.duration_adj.value() * 1000.0) as u64,

            ..config_clone.borrow().clone()
        };

        debug!("Saving settings...");
        *config_clone.borrow_mut() = new_config.clone();
        if let Err(e) = new_config.save() {
            tracing::warn!("Failed to save config: {}", e);
        } else {
            info!("Config saved");
        }
        on_save(new_config);
        window_clone.close();
    });

    window
}

struct KeystrokeWidgets {
    container: GtkBox,
    theme_dropdown: DropDown,
    duration_adj: Adjustment,
    max_keys_adj: Adjustment,
    position_dropdown: DropDown,
    draggable_switch: Switch,
    hotkey_entry: Entry,
}

fn create_keystroke_page(config: &Config) -> KeystrokeWidgets {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .css_classes(["settings-content-area"])
        .build();

    let header = create_page_header("Keystroke", "Configure keystroke display settings");
    container.append(&header);

    add_section_title(&container, "Appearance");

    let appearance_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["settings-card"])
        .build();

    let (theme_row, theme_dropdown) = create_dropdown_row(
        "Theme Style",
        Some("Light, Dark, or follow system theme"),
        &THEME_OPTIONS,
        THEME_OPTIONS
            .iter()
            .position(|&t| t.to_lowercase() == config.keystroke_theme)
            .unwrap_or(2) as u32,
    );
    appearance_card.append(&theme_row);

    let (duration_row, duration_adj) = create_scale_row(
        "Duration",
        Some("How long keystrokes stay visible"),
        config.display_timeout_ms as f64 / 1000.0,
        0.5,
        10.0,
        0.5,
        "s",
    );
    appearance_card.append(&duration_row);

    let (max_keys_row, max_keys_adj) = create_spin_row(
        "Max Keys Displayed",
        Some("Maximum number of keys to show"),
        config.max_keys as f64,
        1.0,
        20.0,
    );
    appearance_card.append(&max_keys_row);

    container.append(&appearance_card);

    add_section_title(&container, "Position & Behavior");

    let position_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["settings-card"])
        .build();

    let current_pos_idx = POSITION_OPTIONS
        .iter()
        .position(|(_, p)| *p == config.position)
        .unwrap_or(4) as u32;
    let (pos_row, position_dropdown) = create_dropdown_row(
        "Screen Position",
        Some("Where keystrokes appear on screen"),
        &POSITION_OPTIONS.map(|(n, _)| n),
        current_pos_idx,
    );
    position_card.append(&pos_row);

    let (drag_row, draggable_switch) = create_switch_row(
        "Draggable",
        Some("Allow dragging to any position"),
        config.keystroke_draggable,
    );
    position_card.append(&drag_row);

    let (hotkey_row, hotkey_entry) = create_entry_row(
        "Trigger Hotkey",
        Some("Keyboard shortcut to toggle"),
        &config.keystroke_hotkey,
    );
    position_card.append(&hotkey_row);

    container.append(&position_card);

    KeystrokeWidgets {
        container,
        theme_dropdown,
        duration_adj,
        max_keys_adj,
        position_dropdown,
        draggable_switch,
        hotkey_entry,
    }
}

struct BubbleWidgets {
    container: GtkBox,
    color_button: ColorDialogButton,
    font_size_adj: Adjustment,
    font_entry: Entry,
    hotkey_entry: Entry,
    sound_switch: Switch,
    position_dropdown: DropDown,
    draggable_switch: Switch,
    duration_adj: Adjustment,
}

fn create_bubble_page(config: &Config) -> BubbleWidgets {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .css_classes(["settings-content-area"])
        .build();

    let header = create_page_header("Bubble", "Configure text bubble display settings");
    container.append(&header);

    add_section_title(&container, "Appearance");

    let appearance_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["settings-card"])
        .build();

    let (color_row, color_button) =
        create_color_row("Bubble Color", Some("Background color of bubbles"), config);
    appearance_card.append(&color_row);

    let (size_row, font_size_adj) = create_scale_row(
        "Font Size",
        Some("Text size in bubbles"),
        config.bubble_font_size,
        0.5,
        3.0,
        0.1,
        "em",
    );
    appearance_card.append(&size_row);

    let (font_row, font_entry) = create_entry_row(
        "Font Family",
        Some("Font used in bubbles"),
        &config.bubble_font_family,
    );
    appearance_card.append(&font_row);

    container.append(&appearance_card);

    add_section_title(&container, "Behavior");

    let behavior_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["settings-card"])
        .build();

    let (hotkey_row, hotkey_entry) = create_entry_row(
        "Trigger Hotkey",
        Some("Keyboard shortcut to trigger bubble"),
        &config.bubble_hotkey,
    );
    behavior_card.append(&hotkey_row);

    let (sound_row, sound_switch) = create_switch_row(
        "Sound Effect",
        Some("Play sound when bubble appears"),
        config.bubble_sound_enabled,
    );
    behavior_card.append(&sound_row);

    let (duration_row, duration_adj) = create_scale_row(
        "Duration",
        Some("How long bubbles stay visible"),
        config.bubble_timeout_ms as f64 / 1000.0,
        1.0,
        60.0,
        1.0,
        "s",
    );
    behavior_card.append(&duration_row);

    container.append(&behavior_card);

    add_section_title(&container, "Position");

    let position_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["settings-card"])
        .build();

    let current_pos_idx = POSITION_OPTIONS
        .iter()
        .position(|(_, p)| *p == config.bubble_position)
        .unwrap_or(2) as u32;
    let (pos_row, position_dropdown) = create_dropdown_row(
        "Screen Position",
        Some("Where bubbles appear on screen"),
        &POSITION_OPTIONS.map(|(n, _)| n),
        current_pos_idx,
    );
    position_card.append(&pos_row);

    let (drag_row, draggable_switch) = create_switch_row(
        "Draggable",
        Some("Allow dragging to any position"),
        config.bubble_draggable,
    );
    position_card.append(&drag_row);

    container.append(&position_card);

    BubbleWidgets {
        container,
        color_button,
        font_size_adj,
        font_entry,
        hotkey_entry,
        sound_switch,
        position_dropdown,
        draggable_switch,
        duration_adj,
    }
}

fn create_page_header(title: &str, subtitle: &str) -> GtkBox {
    let header = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .css_classes(["settings-page-header"])
        .build();

    let icon_box = GtkBox::builder()
        .css_classes(["settings-page-icon"])
        .build();

    let icon_label = Label::builder().label("").build();
    icon_box.append(&icon_label);

    let text_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .valign(gtk4::Align::Center)
        .build();

    let title_label = Label::builder()
        .label(title)
        .css_classes(["settings-page-title"])
        .halign(gtk4::Align::Start)
        .build();

    let subtitle_label = Label::builder()
        .label(subtitle)
        .css_classes(["settings-page-subtitle"])
        .halign(gtk4::Align::Start)
        .build();

    text_box.append(&title_label);
    text_box.append(&subtitle_label);

    header.append(&icon_box);
    header.append(&text_box);

    header
}

fn add_section_title(container: &GtkBox, title: &str) {
    let label = Label::builder()
        .label(title)
        .css_classes(["settings-section-title"])
        .halign(gtk4::Align::Start)
        .build();
    container.append(&label);
}

fn create_dropdown_row(
    label: &str,
    sublabel: Option<&str>,
    options: &[&str],
    selected: u32,
) -> (GtkBox, DropDown) {
    let row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["settings-row"])
        .build();

    let label_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .valign(gtk4::Align::Center)
        .build();

    let label_w = Label::builder()
        .label(label)
        .halign(gtk4::Align::Start)
        .css_classes(["settings-label"])
        .build();
    label_box.append(&label_w);

    if let Some(sub) = sublabel {
        let sub_label = Label::builder()
            .label(sub)
            .halign(gtk4::Align::Start)
            .css_classes(["settings-sublabel"])
            .build();
        label_box.append(&sub_label);
    }

    let list = StringList::new(options);
    let dropdown = DropDown::new(Some(list), gtk4::Expression::NONE);
    dropdown.set_selected(selected);
    dropdown.set_valign(gtk4::Align::Center);
    dropdown.set_halign(gtk4::Align::End);

    row.append(&label_box);
    row.append(&dropdown);
    (row, dropdown)
}

fn create_spin_row(
    label: &str,
    sublabel: Option<&str>,
    value: f64,
    min: f64,
    max: f64,
) -> (GtkBox, Adjustment) {
    let row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["settings-row"])
        .build();

    let label_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .valign(gtk4::Align::Center)
        .build();

    let label_w = Label::builder()
        .label(label)
        .halign(gtk4::Align::Start)
        .css_classes(["settings-label"])
        .build();
    label_box.append(&label_w);

    if let Some(sub) = sublabel {
        let sub_label = Label::builder()
            .label(sub)
            .halign(gtk4::Align::Start)
            .css_classes(["settings-sublabel"])
            .build();
        label_box.append(&sub_label);
    }

    let adj = Adjustment::new(value, min, max, 1.0, 5.0, 0.0);
    let spin = SpinButton::builder()
        .adjustment(&adj)
        .climb_rate(1.0)
        .digits(0)
        .halign(gtk4::Align::End)
        .valign(gtk4::Align::Center)
        .build();

    row.append(&label_box);
    row.append(&spin);
    (row, adj)
}

fn create_switch_row(label: &str, sublabel: Option<&str>, active: bool) -> (GtkBox, Switch) {
    let row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["settings-row"])
        .build();

    let label_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .valign(gtk4::Align::Center)
        .build();

    let label_w = Label::builder()
        .label(label)
        .halign(gtk4::Align::Start)
        .css_classes(["settings-label"])
        .build();
    label_box.append(&label_w);

    if let Some(sub) = sublabel {
        let sub_label = Label::builder()
            .label(sub)
            .halign(gtk4::Align::Start)
            .css_classes(["settings-sublabel"])
            .build();
        label_box.append(&sub_label);
    }

    let switch = Switch::builder()
        .state(active)
        .active(active)
        .valign(gtk4::Align::Center)
        .halign(gtk4::Align::End)
        .build();

    row.append(&label_box);
    row.append(&switch);
    (row, switch)
}

fn create_entry_row(label: &str, sublabel: Option<&str>, text: &str) -> (GtkBox, Entry) {
    let row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["settings-row"])
        .build();

    let label_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .valign(gtk4::Align::Center)
        .build();

    let label_w = Label::builder()
        .label(label)
        .halign(gtk4::Align::Start)
        .css_classes(["settings-label"])
        .build();
    label_box.append(&label_w);

    if let Some(sub) = sublabel {
        let sub_label = Label::builder()
            .label(sub)
            .halign(gtk4::Align::Start)
            .css_classes(["settings-sublabel"])
            .build();
        label_box.append(&sub_label);
    }

    let entry = Entry::builder()
        .text(text)
        .width_request(180)
        .valign(gtk4::Align::Center)
        .halign(gtk4::Align::End)
        .css_classes(["flat-entry"])
        .build();

    row.append(&label_box);
    row.append(&entry);
    (row, entry)
}

fn create_scale_row(
    label: &str,
    sublabel: Option<&str>,
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    _unit: &str,
) -> (GtkBox, Adjustment) {
    let row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["settings-row"])
        .build();

    let label_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .valign(gtk4::Align::Center)
        .build();

    let label_w = Label::builder()
        .label(label)
        .halign(gtk4::Align::Start)
        .css_classes(["settings-label"])
        .build();
    label_box.append(&label_w);

    if let Some(sub) = sublabel {
        let sub_label = Label::builder()
            .label(sub)
            .halign(gtk4::Align::Start)
            .css_classes(["settings-sublabel"])
            .build();
        label_box.append(&sub_label);
    }

    let control_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .halign(gtk4::Align::End)
        .valign(gtk4::Align::Center)
        .build();

    let adj = Adjustment::new(value, min, max, step, step * 2.0, 0.0);
    let scale = Scale::builder()
        .orientation(Orientation::Horizontal)
        .adjustment(&adj)
        .draw_value(true)
        .value_pos(gtk4::PositionType::Left)
        .digits(1)
        .width_request(180)
        .build();

    control_box.append(&scale);

    row.append(&label_box);
    row.append(&control_box);
    (row, adj)
}

fn create_color_row(
    label: &str,
    sublabel: Option<&str>,
    config: &Config,
) -> (GtkBox, ColorDialogButton) {
    let row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["settings-row"])
        .build();

    let label_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .valign(gtk4::Align::Center)
        .build();

    let label_w = Label::builder()
        .label(label)
        .halign(gtk4::Align::Start)
        .css_classes(["settings-label"])
        .build();
    label_box.append(&label_w);

    if let Some(sub) = sublabel {
        let sub_label = Label::builder()
            .label(sub)
            .halign(gtk4::Align::Start)
            .css_classes(["settings-sublabel"])
            .build();
        label_box.append(&sub_label);
    }

    let color_dialog = ColorDialog::new();
    let color_button = ColorDialogButton::new(Some(color_dialog));
    color_button.set_valign(gtk4::Align::Center);
    color_button.set_halign(gtk4::Align::End);

    if let Ok(rgba) = gtk4::gdk::RGBA::parse(&config.bubble_color) {
        color_button.set_rgba(&rgba);
    }

    row.append(&label_box);
    row.append(&color_button);
    (row, color_button)
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
