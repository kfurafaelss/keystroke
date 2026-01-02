use super::{CompositorClient, KeyboardLayouts, LayoutEvent};
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

#[derive(Debug)]
pub struct NiriClient {
    socket_path: String,
}

impl NiriClient {
    #[must_use]
    pub fn new() -> Option<Self> {
        let socket_path = env::var("NIRI_SOCKET")
            .or_else(|_| env::var("NIRI_SOCKET_PATH"))
            .ok()?;

        if std::path::Path::new(&socket_path).exists() {
            Some(Self { socket_path })
        } else {
            tracing::debug!("Niri socket not found at {}", socket_path);
            None
        }
    }

    fn send_request(&self, request: &str) -> anyhow::Result<String> {
        let mut stream = UnixStream::connect(&self.socket_path)?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;

        writeln!(stream, "{}", request)?;
        stream.flush()?;

        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader.read_line(&mut response)?;

        Ok(response)
    }

    fn parse_layouts_response(&self, json: &str) -> KeyboardLayouts {
        let mut layouts = KeyboardLayouts::default();

        if let Some(names) = self.extract_names_array(json) {
            layouts.names = names;
        }

        if let Some(idx) = self.extract_current_idx(json) {
            layouts.current_idx = idx;
        }

        layouts
    }

    fn extract_names_array(&self, json: &str) -> Option<Vec<String>> {
        let key = "\"names\"";
        let key_pos = json.find(key)?;
        let after_key = &json[key_pos + key.len()..];

        let bracket_start = after_key.find('[')?;
        let array_content_start = &after_key[bracket_start + 1..];

        let bracket_end = array_content_start.find(']')?;
        let array_content = &array_content_start[..bracket_end];

        let mut names = Vec::new();
        let mut in_string = false;
        let mut escape_next = false;
        let mut current = String::new();

        for ch in array_content.chars() {
            if escape_next {
                current.push(ch);
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if in_string => {
                    escape_next = true;
                }
                '"' if !in_string => {
                    in_string = true;
                    current.clear();
                }
                '"' if in_string => {
                    in_string = false;
                    if !current.is_empty() {
                        names.push(current.clone());
                    }
                }
                _ if in_string => {
                    current.push(ch);
                }
                _ => {}
            }
        }

        if names.is_empty() {
            None
        } else {
            Some(names)
        }
    }

    fn extract_current_idx(&self, json: &str) -> Option<usize> {
        let key = "\"current_idx\"";
        let key_pos = json.find(key)?;
        let after_key = &json[key_pos + key.len()..];

        let colon_pos = after_key.find(':')?;
        let after_colon = &after_key[colon_pos + 1..];

        let num_str: String = after_colon
            .chars()
            .skip_while(|c| c.is_whitespace())
            .take_while(|c| c.is_ascii_digit())
            .collect();

        num_str.parse().ok()
    }

    pub fn subscribe_events(&self) -> anyhow::Result<BufReader<UnixStream>> {
        let mut stream = UnixStream::connect(&self.socket_path)?;

        writeln!(stream, r#""EventStream""#)?;
        stream.flush()?;

        let mut reader = BufReader::new(stream);

        let mut ack = String::new();
        reader.read_line(&mut ack)?;

        if !ack.contains("\"Ok\"") && !ack.contains("\"Handled\"") {
            anyhow::bail!("Failed to subscribe to Niri events: {}", ack.trim());
        }

        Ok(reader)
    }

    #[must_use]
    pub fn parse_event(&self, line: &str) -> Option<LayoutEvent> {
        if line.contains("\"KeyboardLayoutSwitched\"") {
            if let Some(idx) = self.extract_event_layout_index(line) {
                return Some(LayoutEvent::LayoutSwitched {
                    name: String::new(),
                    index: idx,
                });
            }
        }

        if line.contains("\"KeyboardLayoutsChanged\"") {
            let layouts = self.parse_layouts_response(line);
            if !layouts.is_empty() {
                return Some(LayoutEvent::LayoutsChanged { layouts });
            }
        }

        None
    }

    fn extract_event_layout_index(&self, json: &str) -> Option<usize> {
        let key = "\"idx\"";
        let key_pos = json.find(key)?;
        let after_key = &json[key_pos + key.len()..];

        let colon_pos = after_key.find(':')?;
        let after_colon = &after_key[colon_pos + 1..];

        let num_str: String = after_colon
            .chars()
            .skip_while(|c| c.is_whitespace())
            .take_while(|c| c.is_ascii_digit())
            .collect();

        num_str.parse().ok()
    }
}

impl CompositorClient for NiriClient {
    fn get_keyboard_layouts(&self) -> anyhow::Result<KeyboardLayouts> {
        let response = self.send_request(r#""KeyboardLayouts""#)?;
        Ok(self.parse_layouts_response(&response))
    }

    fn is_available(&self) -> bool {
        std::path::Path::new(&self.socket_path).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_client() -> NiriClient {
        NiriClient {
            socket_path: String::new(),
        }
    }

    #[test]
    fn test_parse_layouts_response() {
        let client = create_test_client();

        let json = r#"{"Ok":{"KeyboardLayouts":{"names":["English (US)","German","French"],"current_idx":1}}}"#;

        let layouts = client.parse_layouts_response(json);
        assert_eq!(layouts.names.len(), 3);
        assert_eq!(layouts.names[0], "English (US)");
        assert_eq!(layouts.names[1], "German");
        assert_eq!(layouts.names[2], "French");
        assert_eq!(layouts.current_idx, 1);
        assert_eq!(layouts.current_name(), Some("German"));
    }

    #[test]
    fn test_parse_layouts_response_single() {
        let client = create_test_client();

        let json = r#"{"Ok":{"KeyboardLayouts":{"names":["English (US)"],"current_idx":0}}}"#;

        let layouts = client.parse_layouts_response(json);
        assert_eq!(layouts.names.len(), 1);
        assert_eq!(layouts.current_idx, 0);
    }

    #[test]
    fn test_extract_names_array() {
        let client = create_test_client();

        let json = r#"{"names":["English","Deutsch","Francais"]}"#;
        let names = client.extract_names_array(json);

        assert!(names.is_some());
        let names = names.unwrap();
        assert_eq!(names.len(), 3);
        assert_eq!(names[0], "English");
        assert_eq!(names[1], "Deutsch");
        assert_eq!(names[2], "Francais");
    }

    #[test]
    fn test_extract_names_with_special_chars() {
        let client = create_test_client();

        let json = r#"{"names":["English (US)","German (Qwertz)"]}"#;
        let names = client.extract_names_array(json);

        assert!(names.is_some());
        let names = names.unwrap();
        assert_eq!(names.len(), 2);
        assert_eq!(names[0], "English (US)");
        assert_eq!(names[1], "German (Qwertz)");
    }

    #[test]
    fn test_extract_current_idx() {
        let client = create_test_client();

        let json = r#"{"current_idx":2}"#;
        let idx = client.extract_current_idx(json);
        assert_eq!(idx, Some(2));
    }

    #[test]
    fn test_parse_event_layout_switched() {
        let client = create_test_client();

        let line = r#"{"Event":{"KeyboardLayoutSwitched":{"idx":1}}}"#;
        let event = client.parse_event(line);

        assert!(event.is_some());
        if let Some(LayoutEvent::LayoutSwitched { index, .. }) = event {
            assert_eq!(index, 1);
        } else {
            panic!("Expected LayoutSwitched event");
        }
    }

    #[test]
    fn test_parse_event_unrelated() {
        let client = create_test_client();

        let line = r#"{"Event":{"WindowFocused":{"id":123}}}"#;
        let event = client.parse_event(line);

        assert!(event.is_none());
    }
}
