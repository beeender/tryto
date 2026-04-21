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
            _ => {
                log::error!("unknown color: '{}'", color);
                String::new()
            }
        }
    }
}

// Preset management

/// Preset definitions: (name, content)
const PRESETS: &[(&str, &str)] = &[
    ("aeroroot", include_str!("presets/aeroroot.toml")),
    ("alacritty", include_str!("presets/alacritty.toml")),
    ("apprentice", include_str!("presets/apprentice.toml")),
    ("ayu-mirage", include_str!("presets/ayu-mirage.toml")),
    ("catppuccin-frappe", include_str!("presets/catppuccin-frappe.toml")),
    ("catppuccin-latte", include_str!("presets/catppuccin-latte.toml")),
    ("catppuccin-macchiato", include_str!("presets/catppuccin-macchiato.toml")),
    ("catppuccin-mocha", include_str!("presets/catppuccin-mocha.toml")),
    ("chiba-dark", include_str!("presets/chiba-dark.toml")),
    ("default", include_str!("presets/default.toml")),
    ("derp", include_str!("presets/derp.toml")),
    ("deus", include_str!("presets/deus.toml")),
    ("dracula", include_str!("presets/dracula.toml")),
    ("dracula-iterm", include_str!("presets/dracula-iterm.toml")),
    ("electrophoretic", include_str!("presets/electrophoretic.toml")),
    ("gruvbox", include_str!("presets/gruvbox.toml")),
    ("gruvbox-dark", include_str!("presets/gruvbox-dark.toml")),
    ("gruvbox-light", include_str!("presets/gruvbox-light.toml")),
    ("hacktober", include_str!("presets/hacktober.toml")),
    ("iterm", include_str!("presets/iterm.toml")),
    ("jetbrains-darcula", include_str!("presets/jetbrains-darcula.toml")),
    ("kitty", include_str!("presets/kitty.toml")),
    ("material-amber", include_str!("presets/material-amber.toml")),
    ("material-design", include_str!("presets/material-design.toml")),
    ("modus-operandi", include_str!("presets/modus-operandi.toml")),
    ("modus-vivendi", include_str!("presets/modus-vivendi.toml")),
    ("modus-vivendi-tinted", include_str!("presets/modus-vivendi-tinted.toml")),
    ("molokai", include_str!("presets/molokai.toml")),
    ("monokai-pro", include_str!("presets/monokai-pro.toml")),
    ("moonfly", include_str!("presets/moonfly.toml")),
    ("neon", include_str!("presets/neon.toml")),
    ("night-owl", include_str!("presets/night-owl.toml")),
    ("nightfly", include_str!("presets/nightfly.toml")),
    ("noirblaze", include_str!("presets/noirblaze.toml")),
    ("nord", include_str!("presets/nord.toml")),
    ("nordiq", include_str!("presets/nordiq.toml")),
    ("nvim", include_str!("presets/nvim.toml")),
    ("nvim-dark", include_str!("presets/nvim-dark.toml")),
    ("nvim-light", include_str!("presets/nvim-light.toml")),
    ("onedark", include_str!("presets/onedark.toml")),
    ("onehalf-dark", include_str!("presets/onehalf-dark.toml")),
    ("panda", include_str!("presets/panda.toml")),
    ("paper-color", include_str!("presets/paper-color.toml")),
    ("paper-color-dark", include_str!("presets/paper-color-dark.toml")),
    ("paper-color-light", include_str!("presets/paper-color-light.toml")),
    ("poimandres", include_str!("presets/poimandres.toml")),
    ("rezza", include_str!("presets/rezza.toml")),
    ("rose-pine", include_str!("presets/rose-pine.toml")),
    ("rose-pine-dawn", include_str!("presets/rose-pine-dawn.toml")),
    ("rose-pine-moon", include_str!("presets/rose-pine-moon.toml")),
    ("selenized", include_str!("presets/selenized.toml")),
    ("selenized-black", include_str!("presets/selenized-black.toml")),
    ("selenized-dark", include_str!("presets/selenized-dark.toml")),
    ("selenized-light", include_str!("presets/selenized-light.toml")),
    ("selenized-white", include_str!("presets/selenized-white.toml")),
    ("solarized", include_str!("presets/solarized.toml")),
    ("solarized-dark", include_str!("presets/solarized-dark.toml")),
    ("solarized-dark-normal-brights", include_str!("presets/solarized-dark-normal-brights.toml")),
    ("solarized-light", include_str!("presets/solarized-light.toml")),
    ("solarized-normal-brights", include_str!("presets/solarized-normal-brights.toml")),
    ("srcery", include_str!("presets/srcery.toml")),
    ("starlight", include_str!("presets/starlight.toml")),
    ("tango", include_str!("presets/tango.toml")),
    ("tempus-autumn", include_str!("presets/tempus-autumn.toml")),
    ("tempus-classic", include_str!("presets/tempus-classic.toml")),
    ("tempus-dawn", include_str!("presets/tempus-dawn.toml")),
    ("tempus-day", include_str!("presets/tempus-day.toml")),
    ("tempus-dusk", include_str!("presets/tempus-dusk.toml")),
    ("tempus-fugit", include_str!("presets/tempus-fugit.toml")),
    ("tempus-future", include_str!("presets/tempus-future.toml")),
    ("tempus-night", include_str!("presets/tempus-night.toml")),
    ("tempus-past", include_str!("presets/tempus-past.toml")),
    ("tempus-rift", include_str!("presets/tempus-rift.toml")),
    ("tempus-spring", include_str!("presets/tempus-spring.toml")),
    ("tempus-summer", include_str!("presets/tempus-summer.toml")),
    ("tempus-tempest", include_str!("presets/tempus-tempest.toml")),
    ("tempus-totus", include_str!("presets/tempus-totus.toml")),
    ("tempus-warp", include_str!("presets/tempus-warp.toml")),
    ("tempus-winter", include_str!("presets/tempus-winter.toml")),
    ("tokyo-night", include_str!("presets/tokyo-night.toml")),
    ("tokyonight-light", include_str!("presets/tokyonight-light.toml")),
    ("tokyonight-night", include_str!("presets/tokyonight-night.toml")),
    ("tokyonight-storm", include_str!("presets/tokyonight-storm.toml")),
    ("visibone", include_str!("presets/visibone.toml")),
    ("xterm", include_str!("presets/xterm.toml")),
    ("zenburn", include_str!("presets/zenburn.toml")),
];

/// Get preset content by name
pub fn get_preset(name: &str) -> Option<&'static str> {
    PRESETS.iter().find(|(n, _)| *n == name).map(|(_, c)| *c)
}

/// List available preset names
pub fn list_presets() -> impl Iterator<Item = &'static str> {
    PRESETS.iter().map(|(n, _)| *n)
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
    fn test_invalid_preset_returns_error() {
        assert!(from_preset("nonexistent").is_err());
    }

    #[test]
    fn test_presets_load() {
        for (name, _) in PRESETS {
            let theme = from_preset(name).expect("preset should load");

            // Helper to check if color is valid hex or named color
            let check_color = |color: &Option<String>, field: &str| {
                if let Some(c) = color {
                    let valid =
                        (c.starts_with('#') && c.len() == 7 && c[1..].chars().all(|ch| ch.is_ascii_hexdigit()))
                        || matches!(c.as_str(), "black" | "red" | "green" | "yellow" | "blue" | "magenta" | "cyan" | "white" | "bright-black" | "bright-red" | "bright-green" | "bright-yellow" | "bright-blue" | "bright-magenta" | "bright-cyan" | "bright-white" | "grey" | "gray");
                    assert!(
                        valid,
                        "preset '{}': field '{}' has invalid color '{}'",
                        name, field, c
                    );
                }
            };

            check_color(&theme.executable.fg, "executable.fg");
            check_color(&theme.argument.fg, "argument.fg");
            check_color(&theme.description.fg, "description.fg");
            check_color(&theme.hint.fg, "hint.fg");
            check_color(&theme.header.fg, "header.fg");
            check_color(&theme.command_line.fg, "command_line.fg");
            check_color(&theme.step_number.fg, "step_number.fg");
            check_color(&theme.prompt.fg, "prompt.fg");
            check_color(&theme.error.fg, "error.fg");
            check_color(&theme.warning.fg, "warning.fg");
            check_color(&theme.success.fg, "success.fg");
        }
    }
}
