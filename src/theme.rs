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

    // Interaction
    pub prompt: ColorDef,

    // Status
    pub error: ColorDef,
    pub warning: ColorDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorDef {
    pub fg: Option<String>,
    pub bg: Option<String>,
    pub bold: bool,
    pub italic: bool,
    pub dimmed: bool,
}

impl Theme {
    pub fn default() -> Self {
        Self {
            executable: ColorDef::new("cyan"),
            argument: ColorDef::new("yellow"),
            description: ColorDef::dimmed(),
            hint: ColorDef::dimmed_italic(),
            header: ColorDef::cyan_bold(),
            command_line: ColorDef::new("green"),
            prompt: ColorDef::new("blue"),
            error: ColorDef::red_bold(),
            warning: ColorDef::new("yellow"),
        }
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

    fn style(&self, def: &ColorDef, s: impl std::fmt::Display) -> String {
        let mut styled = String::new();
        styled.push_str(&s.to_string());

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
}

impl ColorDef {
    fn new(fg: &str) -> Self {
        Self {
            fg: Some(fg.to_string()),
            bg: None,
            bold: false,
            italic: false,
            dimmed: false,
        }
    }

    fn dimmed() -> Self {
        Self {
            fg: None,
            bg: None,
            bold: false,
            italic: false,
            dimmed: true,
        }
    }

    fn dimmed_italic() -> Self {
        Self {
            fg: None,
            bg: None,
            bold: false,
            italic: true,
            dimmed: true,
        }
    }

    fn cyan_bold() -> Self {
        Self {
            fg: Some("cyan".to_string()),
            bg: None,
            bold: true,
            italic: false,
            dimmed: false,
        }
    }

    fn red_bold() -> Self {
        Self {
            fg: Some("red".to_string()),
            bg: None,
            bold: true,
            italic: false,
            dimmed: false,
        }
    }
}
