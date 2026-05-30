use crate::limits::MAX_EVENT_INFO_LENGTH;

pub fn format_event_info(pass_json_pretty: &str, strings_text: Option<&str>) -> String {
  let mut parts = vec![
    "Apple Wallet pass (.pkpass) data:".to_string(),
    String::new(),
    "pass.json:".to_string(),
    pass_json_pretty.to_string(),
  ];

  if let Some(strings_text) = strings_text {
    let trimmed = strings_text.trim();
    if !trimmed.is_empty() {
      let remaining = MAX_EVENT_INFO_LENGTH.saturating_sub(parts.join("\n").len() + 30);
      if remaining > 100 {
        let text = truncate_to_byte_limit(trimmed, remaining);
        parts.push(String::new());
        parts.push("Localization (pass.strings):".to_string());
        parts.push(text);
      }
    }
  }

  let mut event_info = parts.join("\n");
  truncate_on_char_boundary(&mut event_info, MAX_EVENT_INFO_LENGTH);
  event_info
}

fn truncate_on_char_boundary(s: &mut String, max_bytes: usize) {
  if s.len() <= max_bytes {
    return;
  }
  let mut end = max_bytes;
  while end > 0 && !s.is_char_boundary(end) {
    end -= 1;
  }
  s.truncate(end);
}

fn truncate_to_byte_limit(s: &str, max_bytes: usize) -> String {
  if s.len() <= max_bytes {
    return s.to_string();
  }
  let mut end = max_bytes;
  while end > 0 && !s.is_char_boundary(end) {
    end -= 1;
  }
  format!("{}...(truncated)", &s[..end])
}
