use super::{CompositorClient, KeyboardLayouts};
use std::env;
use std::io::{BufReader, Read, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug)]
pub struct HyprlandClient {
    socket_path: PathBuf,

    event_socket_path: PathBuf,
}

impl HyprlandClient {
    #[must_use]
    pub fn new() -> Option<Self> {
        let socket_dir = Self::get_socket_dir()?;

        let socket_path = socket_dir.join(".socket.sock");
        let event_socket_path = socket_dir.join(".socket2.sock");

        if socket_path.exists() {
            Some(Self {
                socket_path,
                event_socket_path,
            })
        } else {
            tracing::debug!("Hyprland socket not found at {:?}", socket_path);
            None
        }
    }

    fn get_socket_dir() -> Option<PathBuf> {
        let runtime_dir = env::var("XDG_RUNTIME_DIR").ok()?;
        let signature = env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;

        let path = PathBuf::from(format!("{}/hypr/{}", runtime_dir, signature));

        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    fn send_command(&self, command: &str) -> anyhow::Result<String> {
        let mut stream = UnixStream::connect(&self.socket_path)?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;

        stream.write_all(command.as_bytes())?;
        stream.shutdown(Shutdown::Write)?;

        let mut response = String::new();
        stream.read_to_string(&mut response)?;

        Ok(response)
    }

    fn parse_devices_response(&self, json: &str) -> KeyboardLayouts {
        let mut layouts = KeyboardLayouts::default();
        let mut seen_layouts = std::collections::HashSet::new();

        let mut search_start = 0;

        while let Some(key_pos) = json[search_start..].find("\"active_keymap\"") {
            let abs_pos = search_start + key_pos;

            if let Some(colon_offset) = json[abs_pos..].find(':') {
                let value_start = abs_pos + colon_offset + 1;

                if let Some(quote_start_offset) = json[value_start..].find('"') {
                    let value_content_start = value_start + quote_start_offset + 1;

                    if let Some(quote_end_offset) = json[value_content_start..].find('"') {
                        let layout_name =
                            &json[value_content_start..value_content_start + quote_end_offset];

                        if !layout_name.is_empty() && seen_layouts.insert(layout_name.to_string()) {
                            layouts.names.push(layout_name.to_string());
                        }

                        search_start = value_content_start + quote_end_offset + 1;
                        continue;
                    }
                }
            }

            search_start = abs_pos + 1;
        }

        layouts
    }

    pub fn subscribe_events(&self) -> anyhow::Result<BufReader<UnixStream>> {
        let stream = UnixStream::connect(&self.event_socket_path)?;

        Ok(BufReader::new(stream))
    }

    #[must_use]
    pub fn parse_event(line: &str) -> Option<(&str, &str)> {
        line.split_once(">>")
    }

    #[must_use]
    pub fn is_layout_event(event_name: &str) -> bool {
        event_name == "activelayout"
    }

    #[must_use]
    pub fn parse_layout_event(data: &str) -> Option<(&str, &str)> {
        data.split_once(',')
    }
}

impl CompositorClient for HyprlandClient {
    fn get_keyboard_layouts(&self) -> anyhow::Result<KeyboardLayouts> {
        let response = self.send_command("j/devices")?;
        Ok(self.parse_devices_response(&response))
    }

    fn is_available(&self) -> bool {
        self.socket_path.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_devices_response() {
        let client = HyprlandClient {
            socket_path: PathBuf::new(),
            event_socket_path: PathBuf::new(),
        };

        let json = r#"{
            "keyboards": [
                {
                    "name": "keyboard1",
                    "active_keymap": "English (US)"
                },
                {
                    "name": "keyboard2",
                    "active_keymap": "German"
                }
            ]
        }"#;

        let layouts = client.parse_devices_response(json);
        assert_eq!(layouts.names.len(), 2);
        assert!(layouts.names.contains(&"English (US)".to_string()));
        assert!(layouts.names.contains(&"German".to_string()));
    }

    #[test]
    fn test_parse_devices_response_duplicates() {
        let client = HyprlandClient {
            socket_path: PathBuf::new(),
            event_socket_path: PathBuf::new(),
        };

        let json = r#"{
            "keyboards": [
                {"active_keymap": "English (US)"},
                {"active_keymap": "English (US)"},
                {"active_keymap": "German"}
            ]
        }"#;

        let layouts = client.parse_devices_response(json);
        assert_eq!(layouts.names.len(), 2);
    }

    #[test]
    fn test_parse_event() {
        let line = "activelayout>>keyboard-name,English (US)";
        let result = HyprlandClient::parse_event(line);
        assert_eq!(result, Some(("activelayout", "keyboard-name,English (US)")));
    }

    #[test]
    fn test_parse_layout_event() {
        let data = "at-keyboard,English (US)";
        let result = HyprlandClient::parse_layout_event(data);
        assert_eq!(result, Some(("at-keyboard", "English (US)")));
    }

    #[test]
    fn test_is_layout_event() {
        assert!(HyprlandClient::is_layout_event("activelayout"));
        assert!(!HyprlandClient::is_layout_event("workspace"));
    }
}
