use crate::input::KeyDisplay;
use evdev::Key;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

const MAX_BUBBLES: usize = 10;

const MAX_CHARS_PER_LINE: usize = 45;

const NEW_BUBBLE_TIMEOUT_MS: u64 = 3000;

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

fn key_to_char(key: Key, display_name: &str) -> Option<BubbleInput> {
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

    if display_name.len() == 1 {
        let c = display_name.chars().next().unwrap();

        return Some(BubbleInput::Char(c.to_ascii_lowercase()));
    }

    match display_name {
        "-" | "=" | "[" | "]" | ";" | "'" | "`" | "\\" | "," | "." | "/" => {
            return Some(BubbleInput::Char(display_name.chars().next().unwrap()));
        }
        _ => {}
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
        }
    }

    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    pub fn process_key(&mut self, key: KeyDisplay) {
        let input = match key_to_char(key.key, &key.display_name) {
            Some(input) => input,
            None => return,
        };

        match input {
            BubbleInput::Char(c) => {
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
                if self.bubbles.back().is_some_and(|b| !b.is_empty()) {
                    self.create_new_bubble();
                }
            }
        }
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

    pub fn has_content(&self) -> bool {
        self.bubbles.iter().any(|b| !b.is_empty())
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
        let result = key_to_char(Key::KEY_A, "A");
        assert!(matches!(result, Some(BubbleInput::Char('a'))));

        let result = key_to_char(Key::KEY_1, "1");
        assert!(matches!(result, Some(BubbleInput::Char('1'))));
    }

    #[test]
    fn test_key_to_char_special_keys() {
        let result = key_to_char(Key::KEY_ENTER, "Enter");
        assert!(matches!(result, Some(BubbleInput::NewLine)));

        let result = key_to_char(Key::KEY_BACKSPACE, "Backspace");
        assert!(matches!(result, Some(BubbleInput::Backspace)));

        let result = key_to_char(Key::KEY_SPACE, "Space");
        assert!(matches!(result, Some(BubbleInput::Char(' '))));
    }

    #[test]
    fn test_key_to_char_modifiers_ignored() {
        assert!(key_to_char(Key::KEY_LEFTCTRL, "Ctrl").is_none());
        assert!(key_to_char(Key::KEY_LEFTALT, "Alt").is_none());
        assert!(key_to_char(Key::KEY_LEFTMETA, "Super").is_none());
    }

    #[test]
    fn test_key_to_char_punctuation() {
        let result = key_to_char(Key::KEY_MINUS, "-");
        assert!(matches!(result, Some(BubbleInput::Char('-'))));

        let result = key_to_char(Key::KEY_DOT, ".");
        assert!(matches!(result, Some(BubbleInput::Char('.'))));
    }
}
