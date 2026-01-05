# Keystroke

![Keystroke Demo](./assets/showcase.gif)

A GTK4 Layer Shell keystroke visualizer for Wayland compositors, built specifically for Wayland. While tools like showmethekey exist, I've always felt they lacked a bit of that "modern" aesthetic. So, heavily inspired by the look of [KeyCastr](https://github.com/keycastr/keycastr), I decided to build my own version.

And yes, it's written in Rust, so you already know it's blazing fast and memory-safe.

## Key Features

- **Wayland Native**: No more X11 workarounds; built to work with Hyprland, Niri, River, Sway, etc.
- **Two Display Modes**:
  - **Keystroke**: The classic view for showing exactly what you're hitting.
  - **Bubble**: A sleek, minimal style inspired by [devaslife's setup](https://www.youtube.com/watch?v=zu_vqAWHy_E).
- **Fully Customizable**: You can tweak the fonts, sizes, and layout to your liking.
- **GTK Theme Support**: It currently pulls from your GTK theme to keep your desktop looking consistent.
- **System Tray Integration**:
  - Quick access to switch modes.
  - Pause/Resume input capture.
  - Access settings.
- **Draggable**: Visualizer windows can be repositioned.

## Supported Compositors

Keystroke automatically detects the running compositor.

- **Full Support** (Layout detection & events):
  - Hyprland
  - Sway
  - Niri
- **Basic Support**:
  - River
  - DWL
  - Labwc
  - Wayfire

## Current State & Contributing

The project is currently in **Early WIP**. It's fully functional, but since it's still in the early stages, there are no official packages (AUR, Nix, etc.) available yet. You'll need to build it from source for now.

I'm mainly sharing this to see if there's interest from the community! If you find this useful, I'd really appreciate it if you could:

- ⭐ Drop a star on the repo: It helps the project get more visibility!
- 💡 Open an Issue: Have a feature idea? Let me know.
- 🛠️ Submit a PR: If you're a fellow Rustacean and want to jump in, contributions are more than welcome.

## Usage

The application launches with a system tray icon (if supported) and a launcher window.

- **Launcher**: Select between Keystroke or Bubble mode.
- **Tray Icon**: Right-click to access the menu to switch modes, pause capture, or quit.
