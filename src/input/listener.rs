use crate::input::device::{discover_keyboards, KeyboardDevice};
use crate::input::keymap::KeyDisplay;
use anyhow::{Context, Result};
use async_channel::{Sender, TrySendError};
use evdev::{Device, InputEventKind, Key};
use nix::poll::{poll, PollFd, PollFlags, PollTimeout};
use std::collections::HashSet;
use std::os::fd::{AsRawFd, BorrowedFd};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use tracing::{error, info, trace, warn};

#[derive(Debug, Clone)]
pub enum KeyEvent {
    Pressed(KeyDisplay),
    Released(KeyDisplay),
    #[allow(dead_code)]
    AllReleased,
}

#[derive(Debug, Clone)]
pub struct ListenerConfig {
    pub all_keyboards: bool,
    pub ignored_keys: HashSet<Key>,
}

impl Default for ListenerConfig {
    fn default() -> Self {
        Self {
            all_keyboards: true,
            ignored_keys: HashSet::new(),
        }
    }
}

pub struct ListenerHandle {
    running: Arc<AtomicBool>,
}

impl Drop for ListenerHandle {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

pub struct KeyListener {
    sender: Sender<KeyEvent>,
    running: Arc<AtomicBool>,
    config: ListenerConfig,
}

impl KeyListener {
    pub fn new(sender: Sender<KeyEvent>, config: ListenerConfig) -> Self {
        Self {
            sender,
            running: Arc::new(AtomicBool::new(false)),
            config,
        }
    }

    pub fn start(&self) -> Result<ListenerHandle> {
        let keyboards = discover_keyboards()?;

        if keyboards.is_empty() {
            anyhow::bail!("No keyboard devices found. Ensure you are in the 'input' group.");
        }

        let devices_to_use: Vec<KeyboardDevice> = if self.config.all_keyboards {
            keyboards
        } else {
            keyboards.into_iter().take(1).collect()
        };

        self.running.store(true, Ordering::SeqCst);

        for keyboard in devices_to_use {
            let sender = self.sender.clone();
            let running = Arc::clone(&self.running);
            let ignored_keys = self.config.ignored_keys.clone();

            thread::spawn(move || {
                if let Err(e) = listen_to_device(keyboard, sender, running, ignored_keys) {
                    error!("Keyboard listener error: {}", e);
                }
            });
        }

        Ok(ListenerHandle {
            running: self.running.clone(),
        })
    }

    #[allow(dead_code)]
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

fn listen_to_device(
    keyboard: KeyboardDevice,
    sender: Sender<KeyEvent>,
    running: Arc<AtomicBool>,
    ignored_keys: HashSet<Key>,
) -> Result<()> {
    let mut device = keyboard.open()?;
    info!("Listening to keyboard: {}", keyboard.name);

    let raw_fd = device.as_raw_fd();

    let borrowed_fd = unsafe { BorrowedFd::borrow_raw(raw_fd) };
    let mut poll_fds = [PollFd::new(borrowed_fd, PollFlags::POLLIN)];
    let mut pressed_keys = HashSet::new();

    while running.load(Ordering::SeqCst) {
        let poll_result = poll(&mut poll_fds, PollTimeout::try_from(100).unwrap());

        match poll_result {
            Ok(_n) => {
                if let Err(e) =
                    process_events(&mut device, &sender, &ignored_keys, &mut pressed_keys)
                {
                    if e.to_string().contains("Channel closed") {
                        info!("Channel closed, stopping listener for {}", keyboard.name);
                        break;
                    }
                    warn!("Error processing events: {}", e);
                }
            }
            Err(e) => {
                error!("Poll error: {}", e);
                break;
            }
        }
    }

    info!("Stopped listening to keyboard: {}", keyboard.name);
    Ok(())
}

fn process_events(
    device: &mut Device,
    sender: &Sender<KeyEvent>,
    ignored_keys: &HashSet<Key>,
    pressed_keys: &mut HashSet<Key>,
) -> Result<()> {
    let events = device.fetch_events().context("Failed to fetch events")?;
    let mut activity = false;

    for event in events {
        if let InputEventKind::Key(key) = event.kind() {
            if ignored_keys.contains(&key) {
                continue;
            }

            activity = true;
            let key_event = match event.value() {
                1 => {
                    trace!("Key pressed: {:?}", key);
                    pressed_keys.insert(key);
                    KeyEvent::Pressed(KeyDisplay::new(key, true))
                }
                0 => {
                    trace!("Key released: {:?}", key);
                    pressed_keys.remove(&key);
                    KeyEvent::Released(KeyDisplay::new(key, false))
                }
                2 => {
                    trace!("Key repeat: {:?}", key);
                    KeyEvent::Pressed(KeyDisplay::new_repeat(key))
                }
                _ => continue,
            };

            if let Err(e) = sender.try_send(key_event) {
                match e {
                    TrySendError::Full(_) => warn!("Channel full, dropping event"),
                    TrySendError::Closed(_) => {
                        return Err(anyhow::anyhow!("Channel closed"));
                    }
                }
            }
        }
    }

    if activity && !pressed_keys.is_empty() {
        if let Ok(actual_state) = device.get_key_state() {
            let stuck_keys: Vec<Key> = pressed_keys
                .iter()
                .filter(|k| !actual_state.contains(**k))
                .cloned()
                .collect();

            for key in stuck_keys {
                trace!("Detected stuck key released (process): {:?}", key);
                pressed_keys.remove(&key);
                let key_display = KeyDisplay::new(key, false);
                let _ = sender.try_send(KeyEvent::Released(key_display));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_listener_config_default() {
        let config = ListenerConfig::default();
        assert!(config.all_keyboards);
        assert!(config.ignored_keys.is_empty());
    }

    #[test]
    fn test_listener_handle_drop() {
        let running = Arc::new(AtomicBool::new(true));
        let handle = ListenerHandle {
            running: running.clone(),
        };

        assert!(running.load(Ordering::SeqCst));
        drop(handle);
        assert!(!running.load(Ordering::SeqCst));
    }
}
