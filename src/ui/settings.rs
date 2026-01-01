use crate::config::{Config, Position};
use gtk4::prelude::*;
use gtk4::{
    Adjustment, Application, ApplicationWindow, Box as GtkBox, Button, DropDown, Label,
    Orientation, Scale, SpinButton, StringList,
};
use gtk4_layer_shell::{Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;
use tracing::{debug, info};

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
        .default_width(400)
        .default_height(500)
        .resizable(true)
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_namespace("keystroke-settings");
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);

    let config_ref = config.borrow();

    let main_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .margin_top(20)
        .margin_bottom(20)
        .margin_start(20)
        .margin_end(20)
        .build();

    let title = Label::builder()
        .label("Settings")
        .css_classes(["title-1"])
        .build();
    main_box.append(&title);

    let position_box = create_section("Position");
    let position_list = StringList::new(&POSITION_OPTIONS.map(|(name, _)| name));
    let position_dropdown = DropDown::new(Some(position_list), gtk4::Expression::NONE);

    let current_pos_idx = POSITION_OPTIONS
        .iter()
        .position(|(_, p)| *p == config_ref.position)
        .unwrap_or(4) as u32;
    position_dropdown.set_selected(current_pos_idx);
    position_dropdown.set_hexpand(true);
    position_box.append(&position_dropdown);
    main_box.append(&position_box);

    let keystroke_timeout_box = create_section("Keystroke Display Timeout (seconds)");
    let keystroke_timeout_adj = Adjustment::new(
        config_ref.display_timeout_ms as f64 / 1000.0,
        0.5,
        10.0,
        0.5,
        1.0,
        0.0,
    );
    let keystroke_timeout_scale = Scale::new(Orientation::Horizontal, Some(&keystroke_timeout_adj));
    keystroke_timeout_scale.set_digits(1);
    keystroke_timeout_scale.set_hexpand(true);
    keystroke_timeout_scale.set_draw_value(true);
    keystroke_timeout_box.append(&keystroke_timeout_scale);
    main_box.append(&keystroke_timeout_box);

    let bubble_timeout_box = create_section("Bubble Display Timeout (seconds)");
    let bubble_timeout_adj = Adjustment::new(
        config_ref.bubble_timeout_ms as f64 / 1000.0,
        1.0,
        60.0,
        1.0,
        5.0,
        0.0,
    );
    let bubble_timeout_scale = Scale::new(Orientation::Horizontal, Some(&bubble_timeout_adj));
    bubble_timeout_scale.set_digits(0);
    bubble_timeout_scale.set_hexpand(true);
    bubble_timeout_scale.set_draw_value(true);
    bubble_timeout_box.append(&bubble_timeout_scale);
    main_box.append(&bubble_timeout_box);

    let max_keys_box = create_section("Maximum Keys Displayed");
    let max_keys_adj = Adjustment::new(config_ref.max_keys as f64, 1.0, 20.0, 1.0, 5.0, 0.0);
    let max_keys_spin = SpinButton::new(Some(&max_keys_adj), 1.0, 0);
    max_keys_spin.set_hexpand(true);
    max_keys_box.append(&max_keys_spin);
    main_box.append(&max_keys_box);

    let margin_box = create_section("Window Margin (pixels)");
    let margin_adj = Adjustment::new(config_ref.margin as f64, 0.0, 200.0, 5.0, 20.0, 0.0);
    let margin_spin = SpinButton::new(Some(&margin_adj), 1.0, 0);
    margin_spin.set_hexpand(true);
    margin_box.append(&margin_spin);
    main_box.append(&margin_box);

    let opacity_box = create_section("Opacity");
    let opacity_adj = Adjustment::new(config_ref.opacity, 0.1, 1.0, 0.1, 0.2, 0.0);
    let opacity_scale = Scale::new(Orientation::Horizontal, Some(&opacity_adj));
    opacity_scale.set_digits(1);
    opacity_scale.set_hexpand(true);
    opacity_scale.set_draw_value(true);
    opacity_box.append(&opacity_scale);
    main_box.append(&opacity_box);

    drop(config_ref);

    let button_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .halign(gtk4::Align::End)
        .margin_top(20)
        .build();

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
        let selected_idx = position_dropdown.selected() as usize;
        let position = POSITION_OPTIONS
            .get(selected_idx)
            .map(|(_, p)| *p)
            .unwrap_or(Position::BottomCenter);

        let new_config = Config {
            position,
            display_timeout_ms: (keystroke_timeout_adj.value() * 1000.0) as u64,
            bubble_timeout_ms: (bubble_timeout_adj.value() * 1000.0) as u64,
            max_keys: max_keys_adj.value() as usize,
            margin: margin_adj.value() as i32,
            opacity: opacity_adj.value(),
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

fn create_section(label: &str) -> GtkBox {
    let section = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .build();

    let label_widget = Label::builder()
        .label(label)
        .halign(gtk4::Align::Start)
        .css_classes(["heading"])
        .build();

    section.append(&label_widget);
    section
}

pub fn show_settings(window: &ApplicationWindow) {
    window.present();
}
