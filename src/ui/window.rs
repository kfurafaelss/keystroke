use crate::config::{Config, Position};
use anyhow::Result;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, CssProvider};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use tracing::info;

fn generate_overlay_css(keystroke_font_size: f64, bubble_font_size: f64) -> String {
    format!(
        r#"
.keystroke-window {{
    background-color: transparent;
    padding: 8px;
    border: none;
    transition: opacity 200ms ease-in-out;
}}

.keystroke-window.fading-out {{
    opacity: 0;
}}

.keystroke-container {{
    padding: 4px;
}}

.keystroke-key {{
    background-color: @card_bg_color;
    color: @card_fg_color;
    border-radius: 9999px;
    padding: 10px 16px;
    margin: 4px;
    font-weight: bold;
    font-size: {keystroke_font_size}em;
    border: 1px solid @borders;
    min-width: 32px;
}}

.keystroke-key label {{
    margin: 0;
    padding: 0;
}}

.keystroke-key.modifier {{
    background-color: @accent_bg_color;
    color: @accent_fg_color;
}}

.keystroke-key.fading {{
}}

.combo-separator {{
    color: inherit;
    font-weight: normal;
    font-size: 0.9em;
}}

.keystroke-separator {{
    color: @window_fg_color;
    font-weight: bold;
    padding: 0 4px;
}}

/* Bubble chat styles */
.bubble-window {{
    background-color: transparent;
    padding: 0;
    border: none;
    transition: opacity 200ms ease-in-out;
}}

.bubble-window.fading-out {{
    opacity: 0;
}}

.bubble-container {{
    padding: 8px;
    background-color: transparent;
}}

.bubble {{
    background-color: @card_bg_color;
    color: @card_fg_color;
    border-radius: 18px;
    padding: 10px 16px;
    margin: 4px 0;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.15);
    font-size: {bubble_font_size}em;
}}
"#,
        keystroke_font_size = keystroke_font_size,
        bubble_font_size = bubble_font_size
    )
}

pub fn create_window(app: &Application, config: &Config) -> Result<ApplicationWindow> {
    let window = ApplicationWindow::builder()
        .application(app)
        .decorated(false)
        .resizable(false)
        .build();

    window.init_layer_shell();

    window.set_layer(Layer::Overlay);

    window.set_namespace("keystroke");

    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::None);

    for (edge, anchor) in config.position.layer_shell_edges() {
        window.set_anchor(edge, anchor);
    }

    window.set_margin(Edge::Top, config.margin);
    window.set_margin(Edge::Bottom, config.margin);
    window.set_margin(Edge::Left, config.margin);
    window.set_margin(Edge::Right, config.margin);

    window.set_exclusive_zone(0);

    apply_css(&window, config);

    window.add_css_class("keystroke-window");

    info!(
        "Created layer shell window at position {:?}",
        config.position
    );

    Ok(window)
}

fn apply_css(window: &ApplicationWindow, config: &Config) {
    let provider = CssProvider::new();
    let css = generate_overlay_css(config.keystroke_font_size, config.bubble_font_size);
    provider.load_from_string(&css);

    let display = gtk4::prelude::WidgetExt::display(window);

    gtk4::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

#[allow(dead_code)]
pub fn update_position(window: &ApplicationWindow, position: Position, margin: i32) {
    for (edge, anchor) in position.layer_shell_edges() {
        window.set_anchor(edge, anchor);
    }

    window.set_margin(Edge::Top, margin);
    window.set_margin(Edge::Bottom, margin);
    window.set_margin(Edge::Left, margin);
    window.set_margin(Edge::Right, margin);
}

pub fn create_bubble_window(app: &Application, config: &Config) -> Result<ApplicationWindow> {
    let window = ApplicationWindow::builder()
        .application(app)
        .decorated(false)
        .resizable(false)
        .build();

    window.init_layer_shell();

    window.set_layer(Layer::Overlay);

    window.set_namespace("keystroke-bubble");

    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::None);

    window.set_anchor(Edge::Top, false);
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, false);

    window.set_margin(Edge::Top, config.margin);
    window.set_margin(Edge::Bottom, config.margin + 100);
    window.set_margin(Edge::Left, config.margin);
    window.set_margin(Edge::Right, config.margin);

    window.set_exclusive_zone(0);

    apply_css(&window, config);

    window.add_css_class("bubble-window");

    info!("Created bubble window");

    Ok(window)
}
