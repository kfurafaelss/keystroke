use crate::config::{Config, Position};
use anyhow::Result;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, CssProvider};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use tracing::info;

const OVERLAY_CSS: &str = r#"
.keystroke-window {
    background-color: @window_bg_color;
    border-radius: 12px;
    padding: 8px 16px;
    border: 1px solid @borders;
}

.keystroke-container {
    padding: 4px;
}

.keystroke-key {
    background-color: @card_bg_color;
    color: @card_fg_color;
    border-radius: 8px;
    padding: 8px 14px;
    margin: 4px;
    font-weight: bold;
    font-size: 1.2em;
    border: 1px solid @borders;
    min-width: 32px;
}

.keystroke-key.modifier {
    background-color: @accent_bg_color;
    color: @accent_fg_color;
}

.keystroke-key.fading {
    opacity: 0.6;
}

.keystroke-separator {
    color: @window_fg_color;
    font-weight: bold;
    padding: 0 4px;
}
"#;

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

    apply_css(&window);

    window.add_css_class("keystroke-window");

    info!(
        "Created layer shell window at position {:?}",
        config.position
    );

    Ok(window)
}

fn apply_css(window: &ApplicationWindow) {
    let provider = CssProvider::new();
    provider.load_from_string(OVERLAY_CSS);

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
