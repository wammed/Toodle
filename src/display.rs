use cosmic::cctk::sctk::reexports::client::protocol::wl_output::WlOutput;
use std::process::Command;
use tracing::warn;

#[derive(Debug, Clone, PartialEq)]
pub struct OutputEntry<O = WlOutput> {
    pub name: Option<String>,
    pub output: O,
    pub logical_size: Option<(u32, u32)>,
    pub scale_factor: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedOutput<O = WlOutput> {
    Active {
        logical_size: (u32, u32),
    },
    Specific {
        name: String,
        output: O,
        logical_size: (u32, u32),
    },
}

impl<O> ResolvedOutput<O> {
    pub fn logical_size(&self) -> (u32, u32) {
        match self {
            ResolvedOutput::Active { logical_size } => *logical_size,
            ResolvedOutput::Specific { logical_size, .. } => *logical_size,
        }
    }

    pub fn output_name(&self) -> Option<&str> {
        match self {
            ResolvedOutput::Active { .. } => None,
            ResolvedOutput::Specific { name, .. } => Some(name.as_str()),
        }
    }

    pub fn specific_output(&self) -> Option<&O> {
        match self {
            ResolvedOutput::Active { .. } => None,
            ResolvedOutput::Specific { output, .. } => Some(output),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OutputManager<O = WlOutput> {
    outputs: Vec<OutputEntry<O>>,
    fallback_resolution: (u32, u32),
    current_resolved: Option<ResolvedOutput<O>>,
}

impl<O: Clone + PartialEq> OutputManager<O> {
    pub fn new(fallback_resolution: (u32, u32)) -> Self {
        Self {
            outputs: Vec::new(),
            fallback_resolution,
            current_resolved: None,
        }
    }

    pub fn outputs(&self) -> &[OutputEntry<O>] {
        &self.outputs
    }

    pub fn current_resolved(&self) -> Option<&ResolvedOutput<O>> {
        self.current_resolved.as_ref()
    }

    pub fn fallback_resolution(&self) -> (u32, u32) {
        self.fallback_resolution
    }

    pub fn set_fallback_resolution(&mut self, res: (u32, u32)) {
        self.fallback_resolution = res;
    }

    pub fn handle_created(&mut self, output: O, name: Option<String>) {
        if let Some(entry) = self.outputs.iter_mut().find(|e| e.output == output) {
            if name.is_some() {
                entry.name = name;
            }
        } else {
            self.outputs.push(OutputEntry {
                name,
                output,
                logical_size: None,
                scale_factor: None,
            });
        }
    }

    pub fn handle_info_update(
        &mut self,
        output: &O,
        name: Option<String>,
        logical_size: Option<(u32, u32)>,
        scale_factor: Option<f64>,
    ) {
        if let Some(entry) = self.outputs.iter_mut().find(|e| &e.output == output) {
            if name.is_some() {
                entry.name = name;
            }
            if logical_size.is_some() {
                entry.logical_size = logical_size;
            }
            if scale_factor.is_some() {
                entry.scale_factor = scale_factor;
            }
        } else {
            self.outputs.push(OutputEntry {
                name,
                output: output.clone(),
                logical_size,
                scale_factor,
            });
        }
    }

    pub fn handle_removed(&mut self, output: &O) {
        self.outputs.retain(|e| &e.output != output);
    }

    /// Resolve target output based on configuration.
    /// If configured with a name and that output exists in available outputs,
    /// returns `ResolvedOutput::Specific`.
    /// Otherwise, falls back to `ResolvedOutput::Active`.
    pub fn resolve(&self, target_config_name: Option<&str>) -> ResolvedOutput<O> {
        if let Some(target) = target_config_name {
            let clean_target = clean_display_name(target);
            if !clean_target.is_empty() {
                for entry in &self.outputs {
                    if let Some(name) = &entry.name {
                        let clean_entry = clean_display_name(name);
                        if clean_entry.eq_ignore_ascii_case(&clean_target) {
                            let size = entry.logical_size.unwrap_or(self.fallback_resolution);
                            return ResolvedOutput::Specific {
                                name: clean_entry,
                                output: entry.output.clone(),
                                logical_size: size,
                            };
                        }
                    }
                }
            }
        }

        // Active output fallback: if any output is connected, its logical size can be used,
        // otherwise use fallback_resolution
        let active_size = self
            .outputs
            .first()
            .and_then(|o| o.logical_size)
            .unwrap_or(self.fallback_resolution);

        ResolvedOutput::Active {
            logical_size: active_size,
        }
    }

    /// Update target resolution and return (newly_resolved, changed).
    /// `changed` is true if the output binding or target geometry has changed.
    pub fn update_target(&mut self, target_config_name: Option<&str>) -> (ResolvedOutput<O>, bool) {
        let new_resolved = self.resolve(target_config_name);
        let changed = self.current_resolved.as_ref() != Some(&new_resolved);
        self.current_resolved = Some(new_resolved.clone());
        (new_resolved, changed)
    }

    /// Set current resolved directly (e.g. after recreating widget)
    pub fn set_current_resolved(&mut self, resolved: ResolvedOutput<O>) {
        self.current_resolved = Some(resolved);
    }

    /// Helper to get target logical resolution without modifying state
    pub fn target_logical_size(&self, target_config_name: Option<&str>) -> (u32, u32) {
        self.resolve(target_config_name).logical_size()
    }
}

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
                if let (Some(w), Some(h)) = (parts.next(), parts.next())
                    && let (Ok(w_val), Ok(h_val)) = (w.parse::<u32>(), h.parse::<u32>())
                {
                    return Some((w_val, h_val));
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

    #[test]
    fn test_output_manager_initial_state_fallback() {
        let mut mgr = OutputManager::<u32>::new((1920, 1080));
        assert_eq!(mgr.fallback_resolution(), (1920, 1080));
        assert!(mgr.outputs().is_empty());

        // Without connected displays, resolves to Active with fallback size
        let (resolved, changed) = mgr.update_target(Some("DP-2"));
        assert!(changed);
        assert_eq!(
            resolved,
            ResolvedOutput::Active {
                logical_size: (1920, 1080)
            }
        );
        assert_eq!(resolved.logical_size(), (1920, 1080));
        assert_eq!(resolved.output_name(), None);
    }

    #[test]
    fn test_output_manager_target_specific_and_geometry() {
        let mut mgr = OutputManager::<u32>::new((1920, 1080));

        // Connect DP-1 (ID 1)
        mgr.handle_created(1, Some("DP-1".to_string()));
        mgr.handle_info_update(&1, Some("DP-1".to_string()), Some((2560, 1440)), Some(1.0));

        // Target is DP-1
        let (resolved, changed) = mgr.update_target(Some("DP-1"));
        assert!(changed);
        assert_eq!(
            resolved,
            ResolvedOutput::Specific {
                name: "DP-1".to_string(),
                output: 1,
                logical_size: (2560, 1440),
            }
        );
        assert_eq!(resolved.logical_size(), (2560, 1440));
        assert_eq!(resolved.output_name(), Some("DP-1"));
        assert_eq!(resolved.specific_output(), Some(&1));

        // Calling update_target again without changes should return changed = false
        let (_, changed_again) = mgr.update_target(Some("DP-1"));
        assert!(!changed_again);
    }

    #[test]
    fn test_output_manager_multimonitor_isolation() {
        let mut mgr = OutputManager::<u32>::new((1920, 1080));

        // Monitor A: DP-1 = 1920x1080
        mgr.handle_created(1, Some("DP-1".to_string()));
        mgr.handle_info_update(&1, Some("DP-1".to_string()), Some((1920, 1080)), Some(1.0));

        // Monitor B: DP-2 = 3840x2160
        mgr.handle_created(2, Some("DP-2".to_string()));
        mgr.handle_info_update(&2, Some("DP-2".to_string()), Some((3840, 2160)), Some(1.0));

        // Target A gets exactly 1920x1080
        let res_a = mgr.resolve(Some("DP-1"));
        assert_eq!(res_a.logical_size(), (1920, 1080));

        // Target B gets exactly 3840x2160
        let res_b = mgr.resolve(Some("DP-2"));
        assert_eq!(res_b.logical_size(), (3840, 2160));

        // Updating target A does not pollute target B
        assert_eq!(mgr.target_logical_size(Some("DP-1")), (1920, 1080));
        assert_eq!(mgr.target_logical_size(Some("DP-2")), (3840, 2160));
    }

    #[test]
    fn test_output_manager_hotplug_attach_and_disconnect() {
        let mut mgr = OutputManager::<u32>::new((1280, 720));

        // User configured target "DP-2", but only DP-1 is connected initially
        mgr.handle_created(1, Some("DP-1".to_string()));
        mgr.handle_info_update(&1, Some("DP-1".to_string()), Some((1920, 1080)), Some(1.0));

        let (init_resolved, _) = mgr.update_target(Some("DP-2"));
        // Falls back to Active because DP-2 is not yet connected
        assert_eq!(
            init_resolved,
            ResolvedOutput::Active {
                logical_size: (1920, 1080)
            }
        );

        // Now DP-2 is hot-plugged!
        mgr.handle_created(2, Some("DP-2".to_string()));
        mgr.handle_info_update(&2, Some("DP-2".to_string()), Some((2560, 1440)), Some(1.0));

        // update_target must detect transition from Active -> Specific
        let (plugged_resolved, changed) = mgr.update_target(Some("DP-2"));
        assert!(changed, "Must detect that DP-2 became available!");
        assert_eq!(
            plugged_resolved,
            ResolvedOutput::Specific {
                name: "DP-2".to_string(),
                output: 2,
                logical_size: (2560, 1440),
            }
        );

        // Now DP-2 is unplugged!
        mgr.handle_removed(&2);

        // update_target must detect transition from Specific -> Active
        let (unplugged_resolved, changed) = mgr.update_target(Some("DP-2"));
        assert!(changed, "Must detect that DP-2 was removed and fall back!");
        assert_eq!(
            unplugged_resolved,
            ResolvedOutput::Active {
                logical_size: (1920, 1080)
            }
        );

        // Now DP-2 is re-connected!
        mgr.handle_created(3, Some("DP-2".to_string()));
        mgr.handle_info_update(&3, Some("DP-2".to_string()), Some((2560, 1440)), Some(1.0));

        // update_target must detect re-connection!
        let (reconnected_resolved, changed) = mgr.update_target(Some("DP-2"));
        assert!(changed, "Must detect that DP-2 reconnected!");
        assert_eq!(
            reconnected_resolved,
            ResolvedOutput::Specific {
                name: "DP-2".to_string(),
                output: 3,
                logical_size: (2560, 1440),
            }
        );
    }
}
