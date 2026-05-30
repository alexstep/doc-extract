const SAMPLE_SIZE: usize = 8192;

pub fn looks_like_utf16(bytes: &[u8]) -> bool {
  if bytes.len() < 4 {
    return false;
  }

  let sample = &bytes[..bytes.len().min(SAMPLE_SIZE)];
  let half = sample.len() / 2;
  if half == 0 {
    return false;
  }

  let even_nuls = sample.iter().step_by(2).filter(|&&b| b == 0).count();
  let odd_nuls = sample.iter().skip(1).step_by(2).filter(|&&b| b == 0).count();

  let even_ratio = even_nuls as f32 / half as f32;
  let odd_ratio = odd_nuls as f32 / half as f32;

  even_ratio > 0.3 || odd_ratio > 0.3
}

pub fn looks_like_text(bytes: &[u8]) -> bool {
  if bytes.is_empty() {
    return true;
  }

  let sample = &bytes[..bytes.len().min(SAMPLE_SIZE)];

  if sample.starts_with(&[0xEF, 0xBB, 0xBF])
    || sample.starts_with(&[0xFF, 0xFE])
    || sample.starts_with(&[0xFE, 0xFF])
  {
    return true;
  }

  let nul_count = sample.iter().filter(|&&b| b == 0).count();
  if nul_count > 0 {
    return looks_like_utf16(sample);
  }

  let valid_utf8 = std::str::from_utf8(sample).is_ok();

  let control_count = sample
    .iter()
    .filter(|&&b| b < 0x20 && b != b'\n' && b != b'\r' && b != b'\t' && b != 0x0C)
    .count();

  let control_ratio = control_count as f32 / sample.len() as f32;
  if control_ratio > 0.02 {
    return false;
  }

  if valid_utf8 {
    return true;
  }

  let printable_or_space = sample
    .iter()
    .filter(|&&b| {
      b == b'\n'
        || b == b'\r'
        || b == b'\t'
        || (0x20..=0x7E).contains(&b)
        || b >= 0x80
    })
    .count();

  let printable_ratio = printable_or_space as f32 / sample.len() as f32;
  printable_ratio > 0.95
}

pub fn looks_obviously_binary(bytes: &[u8]) -> bool {
  if bytes.is_empty() {
    return false;
  }
  !looks_like_text(bytes)
}

pub fn utf16_endian_from_sample(bytes: &[u8]) -> Option<&'static encoding_rs::Encoding> {
  if !looks_like_utf16(bytes) {
    return None;
  }
  let sample = &bytes[..bytes.len().min(SAMPLE_SIZE)];
  let half = sample.len() / 2;
  if half == 0 {
    return None;
  }
  let even_nuls = sample.iter().step_by(2).filter(|&&b| b == 0).count();
  let odd_nuls = sample.iter().skip(1).step_by(2).filter(|&&b| b == 0).count();
  let even_ratio = even_nuls as f32 / half as f32;
  let odd_ratio = odd_nuls as f32 / half as f32;
  if odd_ratio >= even_ratio {
    Some(encoding_rs::UTF_16LE)
  } else {
    Some(encoding_rs::UTF_16BE)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn ascii_is_likely_text() {
    assert!(looks_like_text(b"hello calendar"));
  }

  #[test]
  fn random_binary_is_not_likely_text() {
    let bytes: Vec<u8> = (0..256).map(|i| i as u8).collect();
    assert!(!looks_like_text(&bytes));
  }

  #[test]
  fn utf16le_hello_is_likely_text() {
    let bytes = b"H\0e\0l\0l\0o\0";
    assert!(looks_like_utf16(bytes));
    assert!(looks_like_text(bytes));
  }

  #[test]
  fn empty_is_likely_text() {
    assert!(looks_like_text(b""));
  }
}
