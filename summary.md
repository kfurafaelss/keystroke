Summary for Continuing Development of Keystroke Visualizer
Project Overview
We built Keystroke - a production-grade GTK4 Layer Shell keystroke visualizer for Wayland, inspired by KeyCastr for Mac. It captures keyboard input via evdev and displays pressed keys as an overlay on Wayland compositors (Hyprland, niri, River, Sway, etc.).
Project Location
/home/linuxmobile/Dev/keystroke/
Key Technical Decisions

- Wayland-only - Uses gtk4-layer-shell, no X11/XWayland support
- evdev for input - User must be in input group (no sudo required)
- GTK4 theme integration - CSS uses theme variables like @window_bg_color, @accent_bg_color (no hardcoded colors)
- Modular architecture - Separated into input, ui, config modules for scalability
  Current File Structure
  src/
  ├── main.rs # Entry point, logging setup
  ├── app.rs # Application logic, connects input to UI
  ├── config/
  │ └── mod.rs # TOML config (~/.config/keystroke/config.toml)
  ├── input/
  │ ├── mod.rs # Module exports
  │ ├── device.rs # Keyboard discovery via evdev
  │ ├── keymap.rs # Key code to human-readable name mappings
  │ └── listener.rs # Background thread for keyboard event capture
  └── ui/
  ├── mod.rs # Module exports
  ├── window.rs # GTK4 Layer Shell window setup + CSS
  ├── display.rs # Key display widget with held-key tracking
  └── drag.rs # Drag-to-move functionality
  Key Features Implemented

1. Real-time keystroke visualization - Shows keys as they're pressed
2. Modifier key persistence - Modifiers (Super, Ctrl, Alt, Shift) stay visible while held
3. Draggable overlay - Left-click drag to move anywhere on screen, double-click to reset position
4. Theme integration - Fully opaque, follows GTK4/libadwaita theme
5. Left-aligned keys - Uses Align::Start for key display
6. Max 5 keys - Limited display to 5 simultaneous keys
7. Configurable - Position, timeout, max keys via TOML config
   Recent Changes (Last Session)
8. Removed opacity - overlay is now fully opaque
9. Made overlay freely draggable anywhere (top-left anchoring with margin-based positioning)
10. Changed alignment from center to left (start)
11. Reduced max keys from 8 to 5
12. Fixed modifiers disappearing while held - added is_held tracking
13. Removed excessive key repeat logging spam
    Dependencies (Cargo.toml)
    gtk4 = { version = "0.9", features = ["v4_12"] }
    gtk4-layer-shell = "0.4"
    glib = "0.21"
    evdev = "0.12"
    nix = { version = "0.29", features = ["fs", "poll"] }
    async-channel = "2.3"
    anyhow = "1.0"
    thiserror = "2.0"
    tracing = "0.1"
    tracing-subscriber = { version = "0.3", features = ["env-filter"] }
    serde = { version = "1.0", features = ["derive"] }
    toml = "0.8"
    dirs = "6.0"
    Current State

- All 8 tests pass
- No compiler warnings
- No clippy warnings
- App is functional and ready to use
  How to Run
  cd /home/linuxmobile/Dev/keystroke
  cargo run
  Configuration File Location
  ~/.config/keystroke/config.toml
  position = "bottomcenter"
  display_timeout_ms = 2000
  max_keys = 5
  margin = 20
  show_modifiers = true
  all_keyboards = true
  Potential Future Improvements
- Add CLI arguments for runtime configuration
- Persist drag position across restarts
- Add animation/transitions for key appearance/disappearance
- Add settings UI (preferences window)
- Package for distribution (Nix flake already exists)
- Add mouse button support (currently keyboard-only per user request)
