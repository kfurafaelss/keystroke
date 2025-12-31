use crate::input::{is_modifier, KeyDisplay};
use evdev::Key;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation};
use std::collections::{HashSet, VecDeque};
use std::time::{Duration, Instant};

#[derive(Debug)]
struct DisplayedKey {
    key: Key,

    last_active: Instant,

    is_held: bool,

    label: Label,
}

pub struct KeyDisplayWidget {
    container: GtkBox,

    displayed_keys: VecDeque<DisplayedKey>,

    held_keys: HashSet<Key>,

    max_keys: usize,

    display_duration: Duration,
}

impl KeyDisplayWidget {
    pub fn new(max_keys: usize, display_timeout_ms: u64) -> Self {
        let container = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .halign(gtk4::Align::Start)
            .valign(gtk4::Align::Center)
            .build();

        container.add_css_class("keystroke-container");

        Self {
            container,
            displayed_keys: VecDeque::new(),
            held_keys: HashSet::new(),
            max_keys,
            display_duration: Duration::from_millis(display_timeout_ms),
        }
    }

    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    pub fn add_key(&mut self, key: KeyDisplay) {
        self.held_keys.insert(key.key);

        if let Some(existing) = self.displayed_keys.iter_mut().find(|dk| dk.key == key.key) {
            existing.last_active = Instant::now();
            existing.is_held = true;
            existing.label.remove_css_class("fading");
            return;
        }

        self.remove_expired();

        while self.displayed_keys.len() >= self.max_keys {
            if let Some(old) = self.displayed_keys.pop_front() {
                self.container.remove(&old.label);
            }
        }

        self.cleanup_separators();

        if !self.displayed_keys.is_empty() {
            let separator = Label::new(Some("+"));
            separator.add_css_class("keystroke-separator");
            self.container.append(&separator);
        }

        let label = Label::new(Some(&key.display_name));
        label.add_css_class("keystroke-key");

        if is_modifier(key.key) {
            label.add_css_class("modifier");
        }

        self.container.append(&label);

        let displayed = DisplayedKey {
            key: key.key,
            last_active: Instant::now(),
            is_held: true,
            label,
        };

        self.displayed_keys.push_back(displayed);
    }

    pub fn remove_key(&mut self, key: &KeyDisplay) {
        self.held_keys.remove(&key.key);

        if let Some(displayed) = self.displayed_keys.iter_mut().find(|dk| dk.key == key.key) {
            displayed.is_held = false;
            displayed.last_active = Instant::now();
            displayed.label.add_css_class("fading");
        }
    }

    pub fn remove_expired(&mut self) {
        let now = Instant::now();
        let display_duration = self.display_duration;

        let expired: Vec<usize> = self
            .displayed_keys
            .iter()
            .enumerate()
            .filter(|(_, dk)| !dk.is_held && now.duration_since(dk.last_active) > display_duration)
            .map(|(i, _)| i)
            .collect();

        for &i in expired.iter().rev() {
            if let Some(removed) = self.displayed_keys.remove(i) {
                self.container.remove(&removed.label);
            }
        }

        if !expired.is_empty() {
            self.cleanup_separators();
        }
    }

    pub fn clear(&mut self) {
        while let Some(child) = self.container.first_child() {
            self.container.remove(&child);
        }
        self.displayed_keys.clear();
        self.held_keys.clear();
    }

    fn cleanup_separators(&self) {
        let mut child = self.container.first_child();
        while let Some(widget) = child {
            let next = widget.next_sibling();
            if widget.has_css_class("keystroke-separator") {
                self.container.remove(&widget);
            }
            child = next;
        }

        if self.displayed_keys.len() > 1 {
            let mut child = self.container.first_child();
            while let Some(widget) = child {
                let next = widget.next_sibling();
                if next.is_some() && !widget.has_css_class("keystroke-separator") {
                    let separator = Label::new(Some("+"));
                    separator.add_css_class("keystroke-separator");
                    separator.insert_after(&self.container, Some(&widget));
                }
                child = next;
            }
        }
    }

    #[allow(dead_code)]
    pub fn set_display_timeout(&mut self, timeout_ms: u64) {
        self.display_duration = Duration::from_millis(timeout_ms);
    }

    #[allow(dead_code)]
    pub fn set_max_keys(&mut self, max_keys: usize) {
        self.max_keys = max_keys;

        while self.displayed_keys.len() > max_keys {
            if let Some(old) = self.displayed_keys.pop_front() {
                self.container.remove(&old.label);
            }
        }
    }

    #[allow(dead_code)]
    pub fn has_keys(&self) -> bool {
        !self.displayed_keys.is_empty()
    }
}
