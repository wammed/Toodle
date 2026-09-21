use std::process::Command;
use tracing::warn;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedDisplay {
    pub name: String,
    pub make: Option<String>,
    pub model: Option<String>,
    pub is_enabled: bool,
    pub is_primary: bool,
}

impl DetectedDisplay {
    /// Friendly label for UI display, e.g. "DP-2 (LG Electronics LG ULTRAGEAR)"
    pub fn label(&self) -> String {
        let desc = match (&self.make, &self.model) {
            (Some(make), Some(model)) => format!(" ({} {})", make, model),
            (Some(make), None) => format!(" ({})", make),
            (None, Some(model)) => format!(" ({})", model),
            (None, None) => String::new(),
        };
        format!("{}{}", self.name, desc)
    }
}

/// Strip ANSI escape sequences (colors, bold codes) from a string
pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_escape = false;
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            in_escape = true;
            if chars.peek() == Some(&'[') {
                chars.next();
            }
            continue;
        }
        if in_escape {
            // ANSI escape sequence final characters are in the range 0x40 ('@') to 0x7E ('~')
            if (0x40..=0x7E).contains(&(c as u32)) {
                in_escape = false;
            }
            continue;
        }
        out.push(c);
    }
    out
}

/// Clean a display name string (removes ANSI escapes, trims whitespace)
pub fn clean_display_name(s: &str) -> String {
    strip_ansi(s).trim().to_string()
}

/// Detect connected displays using `cosmic-randr list`
pub fn detect_displays() -> Vec<DetectedDisplay> {
    let output = match Command::new("cosmic-randr").arg("list").output() {
        Ok(out) => out,
        Err(err) => {
            warn!("Failed to run cosmic-randr list: {}", err);
            return Vec::new();
        }
    };

    if !output.status.success() {
        warn!(
            "cosmic-randr list exited with error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Vec::new();
    }

    let text = String::from_utf8_lossy(&output.stdout);
    parse_cosmic_randr_list(&text)
}

/// Detect primary or active screen resolution, defaulting to (2560, 1440) WQHD
pub fn detect_primary_resolution() -> (u32, u32) {
    let output = match Command::new("cosmic-randr").arg("list").output() {
        Ok(out) => out,
        Err(_) => return (2560, 1440),
    };
    if !output.status.success() {
        return (2560, 1440);
    }
    let text = String::from_utf8_lossy(&output.stdout);
    parse_resolution_from_cosmic_randr(&text).unwrap_or((2560, 1440))
}

/// Parse current display resolution from `cosmic-randr list` output
pub fn parse_resolution_from_cosmic_randr(text: &str) -> Option<(u32, u32)> {
    let clean = strip_ansi(text);
    for line in clean.lines() {
        if line.contains("(current)") {
            let trimmed = line.trim();
            if let Some(res_str) = trimmed.split_whitespace().next() {
                let mut parts = res_str.split('x');
                if let (Some(w), Some(h)) = (parts.next(), parts.next()) {
                    if let (Ok(w_val), Ok(h_val)) = (w.parse::<u32>(), h.parse::<u32>()) {
                        return Some((w_val, h_val));
                    }
                }
            }
        }
    }
    None
}

/// Parse output of `cosmic-randr list`
pub fn parse_cosmic_randr_list(text: &str) -> Vec<DetectedDisplay> {
    let clean = strip_ansi(text);
    let mut displays = Vec::new();
    let mut current_display: Option<DetectedDisplay> = None;

    for line in clean.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Top-level display header starts without indentation, e.g.:
        // "DP-1 (enabled)" or "HDMI-A-1 (disabled)"
        if !line.starts_with(' ') && !line.starts_with('\t') {
            if let Some(disp) = current_display.take() {
                displays.push(disp);
            }

            let is_enabled = trimmed.contains("(enabled)");
            let name = trimmed
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim()
                .to_string();

            if !name.is_empty() {
                current_display = Some(DetectedDisplay {
                    name,
                    make: None,
                    model: None,
                    is_enabled,
                    is_primary: false,
                });
            }
        } else if let Some(ref mut disp) = current_display {
            if let Some(stripped) = trimmed.strip_prefix("Make:") {
                disp.make = Some(stripped.trim().to_string());
            } else if let Some(stripped) = trimmed.strip_prefix("Model:") {
                disp.model = Some(stripped.trim().to_string());
            } else if trimmed.starts_with("Xwayland primary: true") {
                disp.is_primary = true;
            }
        }
    }

    if let Some(disp) = current_display {
        displays.push(disp);
    }

    displays
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_ansi() {
        assert_eq!(strip_ansi("\u{1b}[1mDP-1\u{1b}[0m"), "DP-1");
        assert_eq!(strip_ansi("\u{1b}[1;32m(enabled)\u{1b}[0m"), "(enabled)");
        assert_eq!(strip_ansi("Normal text"), "Normal text");
    }

    #[test]
    fn test_clean_display_name() {
        assert_eq!(clean_display_name("  \u{1b}[1mDP-2\u{1b}[0m  "), "DP-2");
        assert_eq!(clean_display_name("DP-1"), "DP-1");
    }

    #[test]
    fn test_parse_cosmic_randr_list_with_ansi() {
        let sample_with_ansi = "\u{1b}[1mDP-2\u{1b}[0m \u{1b}[1;32m(enabled)\u{1b}[0m\u{1b}[1;33m
  Make: \u{1b}[0mLG Electronics\u{1b}[1;33m
  Model: \u{1b}[0mLG ULTRAGEAR\u{1b}[1;33m
  Position: \u{1b}[0m2560,0\u{1b}[1;33m
  Xwayland primary: \u{1b}[0m\u{1b}[31mfalse\u{1b}[0m\u{1b}[1;33m
\u{1b}[1mDP-1\u{1b}[0m \u{1b}[1;32m(enabled)\u{1b}[0m\u{1b}[1;33m
  Make: \u{1b}[0mLG Electronics\u{1b}[1;33m
  Model: \u{1b}[0mLG ULTRAGEAR\u{1b}[1;33m
  Position: \u{1b}[0m0,0\u{1b}[1;33m
  Xwayland primary: \u{1b}[0m\u{1b}[32mtrue\u{1b}[0m\u{1b}[1;33m
";

        let list = parse_cosmic_randr_list(sample_with_ansi);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "DP-2");
        assert_eq!(list[0].make.as_deref(), Some("LG Electronics"));
        assert_eq!(list[0].model.as_deref(), Some("LG ULTRAGEAR"));
        assert!(list[0].is_enabled);
        assert!(!list[0].is_primary);

        assert_eq!(list[1].name, "DP-1");
        assert!(list[1].is_enabled);
        assert!(list[1].is_primary);
    }

    #[test]
    fn test_parse_resolution_from_cosmic_randr() {
        let sample = "DP-1 (enabled)
  Modes:
    2560x1440 @ 119.998 Hz (current) (preferred)
    1920x1080 @ 60.000 Hz
";
        let res = parse_resolution_from_cosmic_randr(sample);
        assert_eq!(res, Some((2560, 1440)));
    }
}
