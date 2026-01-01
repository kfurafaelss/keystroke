use crate::input::KeyDisplay;
use evdev::Key;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

const MAX_BUBBLES: usize = 10;

const MAX_CHARS_PER_LINE: usize = 45;

const NEW_BUBBLE_TIMEOUT_MS: u64 = 3000;

/// Tracks the state of modifier keys
#[derive(Default)]
struct ModifierState {
    shift: bool,
    ctrl: bool,
    alt: bool,
    super_key: bool,
}

impl ModifierState {
    fn update(&mut self, key: Key, pressed: bool) {
        match key {
            Key::KEY_LEFTSHIFT | Key::KEY_RIGHTSHIFT => self.shift = pressed,
            Key::KEY_LEFTCTRL | Key::KEY_RIGHTCTRL => self.ctrl = pressed,
            Key::KEY_LEFTALT | Key::KEY_RIGHTALT => self.alt = pressed,
            Key::KEY_LEFTMETA | Key::KEY_RIGHTMETA => self.super_key = pressed,
            _ => {}
        }
    }

    /// Returns true if any "command" modifier is held (ctrl, alt, super)
    /// These typically indicate shortcuts, not typing
    fn has_command_modifier(&self) -> bool {
        self.ctrl || self.alt || self.super_key
    }
}

struct ChatBubble {
    text: String,

    last_modified: Instant,

    label: Label,
}

impl ChatBubble {
    fn new() -> Self {
        let label = Label::builder()
            .wrap(true)
            .wrap_mode(gtk4::pango::WrapMode::WordChar)
            .max_width_chars(MAX_CHARS_PER_LINE as i32)
            .xalign(0.0)
            .halign(gtk4::Align::Start)
            .build();
        label.add_css_class("bubble");

        Self {
            text: String::new(),
            last_modified: Instant::now(),
            label,
        }
    }

    fn append_char(&mut self, c: char) {
        self.text.push(c);
        self.label.set_text(&self.text);
        self.last_modified = Instant::now();
    }

    fn backspace(&mut self) {
        self.text.pop();
        self.label.set_text(&self.text);
        self.last_modified = Instant::now();
    }

    fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    fn widget(&self) -> &Label {
        &self.label
    }

    fn is_stale(&self, timeout: Duration) -> bool {
        self.last_modified.elapsed() > timeout
    }
}

fn is_ignored_key(key: Key) -> bool {
    matches!(
        key,
        Key::KEY_LEFTCTRL
            | Key::KEY_RIGHTCTRL
            | Key::KEY_LEFTALT
            | Key::KEY_RIGHTALT
            | Key::KEY_LEFTMETA
            | Key::KEY_RIGHTMETA
            | Key::KEY_LEFTSHIFT
            | Key::KEY_RIGHTSHIFT
            | Key::KEY_CAPSLOCK
            | Key::KEY_NUMLOCK
            | Key::KEY_SCROLLLOCK
            | Key::KEY_FN
            | Key::KEY_ESC
            | Key::KEY_INSERT
            | Key::KEY_HOME
            | Key::KEY_END
            | Key::KEY_PAGEUP
            | Key::KEY_PAGEDOWN
            | Key::KEY_UP
            | Key::KEY_DOWN
            | Key::KEY_LEFT
            | Key::KEY_RIGHT
            | Key::KEY_PRINT
            | Key::KEY_PAUSE
            | Key::KEY_F1
            | Key::KEY_F2
            | Key::KEY_F3
            | Key::KEY_F4
            | Key::KEY_F5
            | Key::KEY_F6
            | Key::KEY_F7
            | Key::KEY_F8
            | Key::KEY_F9
            | Key::KEY_F10
            | Key::KEY_F11
            | Key::KEY_F12
    )
}

fn key_to_char(key: Key, display_name: &str, shift_held: bool) -> Option<BubbleInput> {
    match key {
        Key::KEY_ENTER | Key::KEY_KPENTER => return Some(BubbleInput::NewLine),
        Key::KEY_BACKSPACE => return Some(BubbleInput::Backspace),
        Key::KEY_DELETE => return Some(BubbleInput::Backspace),
        Key::KEY_SPACE => return Some(BubbleInput::Char(' ')),
        Key::KEY_TAB => return Some(BubbleInput::Char(' ')),
        _ => {}
    }

    if is_ignored_key(key) {
        return None;
    }

    // Handle single character keys (letters and numbers)
    if display_name.len() == 1 {
        let c = display_name.chars().next().unwrap();

        // For letters, respect shift state
        if c.is_ascii_alphabetic() {
            let c = if shift_held {
                c.to_ascii_uppercase()
            } else {
                c.to_ascii_lowercase()
            };
            return Some(BubbleInput::Char(c));
        }

        // For numbers with shift, show the shifted symbol
        if c.is_ascii_digit() {
            if shift_held {
                let shifted = match c {
                    '1' => '!',
                    '2' => '@',
                    '3' => '#',
                    '4' => '$',
                    '5' => '%',
                    '6' => '^',
                    '7' => '&',
                    '8' => '*',
                    '9' => '(',
                    '0' => ')',
                    _ => c,
                };
                return Some(BubbleInput::Char(shifted));
            }
            return Some(BubbleInput::Char(c));
        }

        // Handle single-character punctuation with shift
        if shift_held {
            let shifted = match c {
                '-' => '_',
                '=' => '+',
                '[' => '{',
                ']' => '}',
                ';' => ':',
                '\'' => '"',
                '`' => '~',
                '\\' => '|',
                ',' => '<',
                '.' => '>',
                '/' => '?',
                _ => c,
            };
            return Some(BubbleInput::Char(shifted));
        }

        return Some(BubbleInput::Char(c));
    }

    None
}

#[derive(Debug)]
enum BubbleInput {
    Char(char),
    Backspace,
    NewLine,
}

pub struct BubbleDisplayWidget {
    container: GtkBox,

    bubbles: VecDeque<ChatBubble>,

    display_duration: Duration,

    new_bubble_timeout: Duration,

    /// Tracks current modifier key state
    modifiers: ModifierState,

    /// Set to true when Enter is pressed - next char will start a new bubble
    want_new_bubble: bool,
}

impl BubbleDisplayWidget {
    pub fn new(display_timeout_ms: u64) -> Self {
        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(6)
            .halign(gtk4::Align::Start)
            .valign(gtk4::Align::End)
            .build();

        container.add_css_class("bubble-container");

        Self {
            container,
            bubbles: VecDeque::new(),
            display_duration: Duration::from_millis(display_timeout_ms),
            new_bubble_timeout: Duration::from_millis(NEW_BUBBLE_TIMEOUT_MS),
            modifiers: ModifierState::default(),
            want_new_bubble: false,
        }
    }

    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    /// Process a key press event
    pub fn process_key(&mut self, key: KeyDisplay) {
        // Update modifier state for press
        self.modifiers.update(key.key, true);

        // If a command modifier (ctrl/alt/super) is held, ignore the key
        // This prevents showing 'h', 'j', 'k', 'l' when using Super+hjkl for window management
        if self.modifiers.has_command_modifier() {
            return;
        }

        let input = match key_to_char(key.key, &key.display_name, self.modifiers.shift) {
            Some(input) => input,
            None => return,
        };

        match input {
            BubbleInput::Char(c) => {
                // If we wanted a new bubble (from Enter), create it now
                if self.want_new_bubble {
                    self.want_new_bubble = false;
                    if self.bubbles.back().is_some_and(|b| !b.is_empty()) {
                        self.create_new_bubble();
                    }
                }
                self.ensure_active_bubble();
                if let Some(bubble) = self.bubbles.back_mut() {
                    bubble.append_char(c);
                }
            }
            BubbleInput::Backspace => {
                if let Some(bubble) = self.bubbles.back_mut() {
                    if bubble.is_empty() {
                        if let Some(removed) = self.bubbles.pop_back() {
                            self.container.remove(removed.widget());
                        }

                        if let Some(prev_bubble) = self.bubbles.back_mut() {
                            prev_bubble.backspace();
                        }
                    } else {
                        bubble.backspace();
                    }
                }
            }
            BubbleInput::NewLine => {
                // Don't create empty bubble - just set flag for next character
                if self.bubbles.back().is_some_and(|b| !b.is_empty()) {
                    self.want_new_bubble = true;
                }
            }
        }
    }

    /// Process a key release event to track modifier state
    pub fn process_key_release(&mut self, key: KeyDisplay) {
        self.modifiers.update(key.key, false);
    }

    fn ensure_active_bubble(&mut self) {
        let need_new_bubble = if let Some(bubble) = self.bubbles.back() {
            bubble.is_stale(self.new_bubble_timeout) && !bubble.is_empty()
        } else {
            true
        };

        if need_new_bubble {
            self.create_new_bubble();
        }
    }

    fn create_new_bubble(&mut self) {
        while self.bubbles.len() >= MAX_BUBBLES {
            if let Some(removed) = self.bubbles.pop_front() {
                self.container.remove(removed.widget());
            }
        }

        let bubble = ChatBubble::new();
        self.container.append(bubble.widget());
        self.bubbles.push_back(bubble);
    }

    pub fn remove_expired(&mut self) {
        let duration = self.display_duration;

        while self.bubbles.len() > 1 {
            if let Some(bubble) = self.bubbles.front() {
                if bubble.last_modified.elapsed() > duration {
                    if let Some(removed) = self.bubbles.pop_front() {
                        self.container.remove(removed.widget());
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        while let Some(bubble) = self.bubbles.pop_front() {
            self.container.remove(bubble.widget());
        }
    }

    #[allow(dead_code)]
    pub fn has_content(&self) -> bool {
        self.bubbles.iter().any(|b| !b.is_empty())
    }

    /// Returns true if the widget should be visible (has non-stale content)
    pub fn should_show(&self) -> bool {
        if self.bubbles.is_empty() {
            return false;
        }

        // Check if any bubble has content and is not stale
        self.bubbles.iter().any(|b| {
            !b.is_empty() && !b.is_stale(self.display_duration)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ignored_key() {
        assert!(is_ignored_key(Key::KEY_LEFTCTRL));
        assert!(is_ignored_key(Key::KEY_RIGHTALT));
        assert!(is_ignored_key(Key::KEY_LEFTMETA));
        assert!(is_ignored_key(Key::KEY_F1));
        assert!(!is_ignored_key(Key::KEY_A));
        assert!(!is_ignored_key(Key::KEY_SPACE));
    }

    #[test]
    fn test_key_to_char_regular_keys() {
        // Without shift - lowercase
        let result = key_to_char(Key::KEY_A, "A", false);
        assert!(matches!(result, Some(BubbleInput::Char('a'))));

        // With shift - uppercase
        let result = key_to_char(Key::KEY_A, "A", true);
        assert!(matches!(result, Some(BubbleInput::Char('A'))));

        let result = key_to_char(Key::KEY_1, "1", false);
        assert!(matches!(result, Some(BubbleInput::Char('1'))));

        // Shift + 1 = !
        let result = key_to_char(Key::KEY_1, "1", true);
        assert!(matches!(result, Some(BubbleInput::Char('!'))));
    }

    #[test]
    fn test_key_to_char_special_keys() {
        let result = key_to_char(Key::KEY_ENTER, "Enter", false);
        assert!(matches!(result, Some(BubbleInput::NewLine)));

        let result = key_to_char(Key::KEY_BACKSPACE, "Backspace", false);
        assert!(matches!(result, Some(BubbleInput::Backspace)));

        let result = key_to_char(Key::KEY_SPACE, "Space", false);
        assert!(matches!(result, Some(BubbleInput::Char(' '))));
    }

    #[test]
    fn test_key_to_char_modifiers_ignored() {
        assert!(key_to_char(Key::KEY_LEFTCTRL, "Ctrl", false).is_none());
        assert!(key_to_char(Key::KEY_LEFTALT, "Alt", false).is_none());
        assert!(key_to_char(Key::KEY_LEFTMETA, "Super", false).is_none());
    }

    #[test]
    fn test_key_to_char_punctuation() {
        // Without shift
        let result = key_to_char(Key::KEY_MINUS, "-", false);
        assert!(matches!(result, Some(BubbleInput::Char('-'))));

        let result = key_to_char(Key::KEY_DOT, ".", false);
        assert!(matches!(result, Some(BubbleInput::Char('.'))));

        // With shift
        let result = key_to_char(Key::KEY_MINUS, "-", true);
        assert!(matches!(result, Some(BubbleInput::Char('_'))));

        let result = key_to_char(Key::KEY_DOT, ".", true);
        assert!(matches!(result, Some(BubbleInput::Char('>'))));
    }
}
