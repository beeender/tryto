use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    // Command components
    pub executable: ColorDef,
    pub argument: ColorDef,

    // Descriptions
    pub description: ColorDef,
    pub hint: ColorDef,

    // Output elements
    pub header: ColorDef,
    pub command_line: ColorDef,
    pub step_number: ColorDef,

    // Interaction
    pub prompt: ColorDef,

    // Status
    pub error: ColorDef,
    pub warning: ColorDef,
    pub success: ColorDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorDef {
    #[serde(default)]
    pub fg: Option<String>,
    #[serde(default)]
    pub bg: Option<String>,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
    #[serde(default)]
    pub dimmed: bool,
}

/// Error type for preset loading
#[derive(Debug)]
pub enum PresetError {
    NotFound(String),
    ParseError(String),
}

impl std::fmt::Display for PresetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PresetError::NotFound(name) => write!(f, "preset '{}' not found", name),
            PresetError::ParseError(msg) => write!(f, "failed to parse preset: {}", msg),
        }
    }
}

impl std::error::Error for PresetError {}

impl Theme {
    pub fn default() -> Self {
        from_preset("default").expect("failed to load default theme preset")
    }

    pub fn executable(&self, s: impl std::fmt::Display) -> String {
        self.style(&self.executable, s)
    }

    pub fn argument(&self, s: impl std::fmt::Display) -> String {
        self.style(&self.argument, s)
    }

    pub fn description(&self, s: impl std::fmt::Display) -> String {
        self.style(&self.description, s)
    }

    pub fn hint(&self, s: impl std::fmt::Display) -> String {
        self.style(&self.hint, s)
    }

    pub fn header(&self, s: impl std::fmt::Display) -> String {
        self.style(&self.header, s)
    }

    pub fn command_line(&self, s: impl std::fmt::Display) -> String {
        self.style(&self.command_line, s)
    }

    pub fn prompt(&self, s: impl std::fmt::Display) -> String {
        self.style(&self.prompt, s)
    }

    pub fn error(&self, s: impl std::fmt::Display) -> String {
        self.style(&self.error, s)
    }

    pub fn warning(&self, s: impl std::fmt::Display) -> String {
        self.style(&self.warning, s)
    }

    pub fn success(&self, s: impl std::fmt::Display) -> String {
        self.style(&self.success, s)
    }

    pub fn step_number(&self, s: impl std::fmt::Display) -> String {
        self.style(&self.step_number, s)
    }

    fn style(&self, def: &ColorDef, s: impl std::fmt::Display) -> String {
        let mut styled = s.to_string();

        // Apply foreground color first
        if let Some(fg) = &def.fg {
            styled = Self::apply_color(styled, fg);
        }

        // Apply styles
        if def.bold {
            styled = styled.bold().to_string();
        }
        if def.italic {
            styled = styled.italic().to_string();
        }
        if def.dimmed {
            styled = styled.dimmed().to_string();
        }

        styled
    }

    fn apply_color(s: String, color: &str) -> String {
        // Try hex color first (#RRGGBB)
        if color.starts_with('#')
            && color.len() == 7
            && let Ok(r) = u8::from_str_radix(&color[1..3], 16)
            && let Ok(g) = u8::from_str_radix(&color[3..5], 16)
            && let Ok(b) = u8::from_str_radix(&color[5..7], 16)
        {
            return s.truecolor(r, g, b).to_string();
        }

        // Named colors
        match color.to_lowercase().as_str() {
            "black" => s.black().to_string(),
            "red" => s.red().to_string(),
            "green" => s.green().to_string(),
            "yellow" => s.yellow().to_string(),
            "blue" => s.blue().to_string(),
            "magenta" => s.magenta().to_string(),
            "cyan" => s.cyan().to_string(),
            "white" => s.white().to_string(),
            "bright-black" | "grey" | "gray" => s.bright_black().to_string(),
            "bright-red" => s.bright_red().to_string(),
            "bright-green" => s.bright_green().to_string(),
            "bright-yellow" => s.bright_yellow().to_string(),
            "bright-blue" => s.bright_blue().to_string(),
            "bright-magenta" => s.bright_magenta().to_string(),
            "bright-cyan" => s.bright_cyan().to_string(),
            "bright-white" => s.bright_white().to_string(),
            _ => s, // Unknown color, return as-is
        }
    }
}

// Preset management

/// Default theme preset embedded at compile time
const PRESET_DEFAULT: &str = include_str!("presets/default.toml");

/// Tokyo Night theme preset embedded at compile time
const PRESET_TOKYO_NIGHT: &str = include_str!("presets/tokyo-night.toml");

/// Get preset content by name
pub fn get_preset(name: &str) -> Option<&'static str> {
    match name {
        "default" => Some(PRESET_DEFAULT),
        "tokyo-night" => Some(PRESET_TOKYO_NIGHT),
        _ => None,
    }
}

/// List available preset names
pub fn list_presets() -> &'static [&'static str] {
    &["default", "tokyo-night"]
}

/// Load theme from preset name
pub fn from_preset(name: &str) -> Result<Theme, PresetError> {
    let toml_str = get_preset(name)
        .ok_or_else(|| PresetError::NotFound(name.to_string()))?;

    toml::from_str(toml_str)
        .map_err(|e| PresetError::ParseError(e.to_string()))
}

/// Load theme from config or use default
pub fn load(theme_name: Option<&str>) -> Result<Theme, PresetError> {
    match theme_name {
        Some(name) => from_preset(name),
        None => from_preset("default"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presets_load() {
        assert!(from_preset("default").is_ok());
        assert!(from_preset("tokyo-night").is_ok());
    }

    #[test]
    fn test_invalid_preset_returns_error() {
        assert!(from_preset("nonexistent").is_err());
    }
}
