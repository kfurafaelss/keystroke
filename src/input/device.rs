use anyhow::{Context, Result};
use evdev::Device;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct KeyboardDevice {
    pub path: PathBuf,

    pub name: String,
}

impl KeyboardDevice {
    pub fn open(&self) -> Result<Device> {
        Device::open(&self.path).with_context(|| format!("Failed to open device: {:?}", self.path))
    }
}

pub fn discover_keyboards() -> Result<Vec<KeyboardDevice>> {
    let mut keyboards = Vec::new();
    let input_dir = PathBuf::from("/dev/input");

    let entries = fs::read_dir(&input_dir)
        .with_context(|| format!("Failed to read directory: {:?}", input_dir))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();

        if !file_name.starts_with("event") {
            continue;
        }

        match Device::open(&path) {
            Ok(device) => {
                if is_keyboard(&device) {
                    let name = device.name().unwrap_or("Unknown Keyboard").to_string();

                    info!("Found keyboard: {} at {:?}", name, path);

                    keyboards.push(KeyboardDevice { path, name });
                }
            }
            Err(e) => {
                debug!("Could not open {:?}: {}", path, e);
            }
        }
    }

    if keyboards.is_empty() {
        warn!("No keyboard devices found. Ensure you are in the 'input' group.");
    }

    Ok(keyboards)
}

fn is_keyboard(device: &Device) -> bool {
    let supported = device.supported_events();
    if !supported.contains(evdev::EventType::KEY) {
        return false;
    }

    if let Some(keys) = device.supported_keys() {
        let has_letter_keys = keys.contains(evdev::Key::KEY_A)
            && keys.contains(evdev::Key::KEY_Z)
            && keys.contains(evdev::Key::KEY_SPACE);

        return has_letter_keys;
    }

    false
}

#[allow(dead_code)]
pub fn get_primary_keyboard() -> Result<KeyboardDevice> {
    let keyboards = discover_keyboards()?;

    let physical: Vec<_> = keyboards
        .iter()
        .filter(|kb| !kb.name.to_lowercase().contains("virtual"))
        .cloned()
        .collect();

    if let Some(keyboard) = physical.into_iter().next() {
        return Ok(keyboard);
    }

    keyboards
        .into_iter()
        .next()
        .context("No keyboard devices found")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_returns_result() {
        let result = discover_keyboards();
        assert!(result.is_ok());
    }
}
