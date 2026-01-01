use gtk4::prelude::*;
use gtk4::{ApplicationWindow, GestureClick, GestureDrag};
use gtk4_layer_shell::{Edge, LayerShell};
use std::cell::Cell;
use std::rc::Rc;
use tracing::debug;

const DEFAULT_MARGIN_X: i32 = 20;
const DEFAULT_MARGIN_Y: i32 = 20;

#[derive(Debug)]
struct DragState {
    start_x: Cell<i32>,
    start_y: Cell<i32>,
}

impl Default for DragState {
    fn default() -> Self {
        Self {
            start_x: Cell::new(DEFAULT_MARGIN_X),
            start_y: Cell::new(DEFAULT_MARGIN_Y),
        }
    }
}

pub fn setup_drag(window: &ApplicationWindow) {
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Bottom, false);
    window.set_anchor(Edge::Right, false);

    window.set_margin(Edge::Left, DEFAULT_MARGIN_X);
    window.set_margin(Edge::Top, DEFAULT_MARGIN_Y);

    let drag_state = Rc::new(DragState::default());

    let gesture = GestureDrag::new();
    gesture.set_button(1);

    let state = Rc::clone(&drag_state);
    let win = window.clone();
    gesture.connect_drag_begin(move |_, _, _| {
        let current_margin_x = win.margin(Edge::Left);
        let current_margin_y = win.margin(Edge::Top);

        state.start_x.set(current_margin_x);
        state.start_y.set(current_margin_y);

        debug!(
            "Drag started at margin ({}, {})",
            current_margin_x, current_margin_y
        );
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

    gesture.connect_drag_end(move |_, offset_x, offset_y| {
        debug!("Drag ended with offset ({}, {})", offset_x, offset_y);
    });

    window.add_controller(gesture);

    let click = GestureClick::new();
    click.set_button(1);
    let win = window.clone();
    click.connect_released(move |_gesture, n_press, _, _| {
        if n_press == 2 {
            debug!("Double-click: resetting position to default");
            win.set_margin(Edge::Left, DEFAULT_MARGIN_X);
            win.set_margin(Edge::Top, DEFAULT_MARGIN_Y);
        }
    });
    window.add_controller(click);
}
