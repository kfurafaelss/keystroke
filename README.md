# Keystroke

A GTK4 Layer Shell keystroke visualizer for Wayland compositors.

## Features

- **Wayland Native**: Built with GTK4 and Layer Shell for seamless Wayland integration.
- **Multiple Modes**:
  - **Keystroke Mode**: Displays a history of typed keys.
  - **Bubble Mode**: Shows pressed keys in a floating bubble.
- **Compositor Support**: Auto-detection for Hyprland, Sway, Niri, River, DWL, Labwc, and Wayfire.
  - Advanced layout detection available for Hyprland, Sway, and Niri.
- **Input Handling**: Captures input directly using `evdev`, supporting multiple keyboards.
- **System Tray**:
  - Quick access to switch modes.
  - Pause/Resume input capture.
  - Access settings.
- **Customizable**: Configurable behavior and appearance.
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

## Usage

The application launches with a system tray icon (if supported) and a launcher window.

- **Launcher**: Select between Keystroke or Bubble mode.
- **Tray Icon**: Right-click to access the menu to switch modes, pause capture, or quit.
