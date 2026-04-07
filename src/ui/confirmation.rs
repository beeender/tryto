/// Danger level for command execution confirmation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DangerLevel {
    /// Level 0: Safe operation (Y/n prompt)
    Safe = 0,
    /// Level 1: Caution - requires typing "yes"
    Caution = 1,
    /// Level 2: Dangerous - requires "yes <3-char code>" (numbers + lowercase)
    Dangerous = 2,
    /// Level 3: Critical - requires "yes <6-char code>" (numbers + lower + upper)
    Critical = 3,
}

impl DangerLevel {
    /// Create from u8, clamping to valid range 0-3
    pub fn from_u8(level: u8) -> Self {
        match level {
            0 => Self::Safe,
            1 => Self::Caution,
            2 => Self::Dangerous,
            3.. => Self::Critical,
        }
    }

    /// Get the confirmation prompt text
    pub fn prompt(&self, code: Option<&str>) -> String {
        match self {
            DangerLevel::Safe => "Execute? [Y/n]".to_string(),
            DangerLevel::Caution => "Type 'yes' to confirm execution: ".to_string(),
            DangerLevel::Dangerous => {
                format!("Type 'yes {}' to confirm execution: ", code.unwrap_or("???"))
            }
            DangerLevel::Critical => {
                format!("CRITICAL: Type 'yes {}' to confirm execution: ", code.unwrap_or("???"))
            }
        }
    }
}

/// Confirmation with danger level and optional code
#[derive(Debug, Clone)]
pub struct Confirmation {
    pub level: DangerLevel,
    pub code: Option<String>,
}

impl Confirmation {
    /// Create a safe confirmation (level 0)
    pub fn safe() -> Self {
        Self {
            level: DangerLevel::Safe,
            code: None,
        }
    }

    /// Create a caution confirmation (level 1)
    pub fn caution() -> Self {
        Self {
            level: DangerLevel::Caution,
            code: None,
        }
    }

    /// Create a dangerous confirmation (level 2) with 3-char code
    pub fn dangerous() -> Self {
        Self {
            level: DangerLevel::Dangerous,
            code: Some(Self::generate_code(2)),
        }
    }

    /// Create a critical confirmation (level 3) with 6-char code
    pub fn critical() -> Self {
        Self {
            level: DangerLevel::Critical,
            code: Some(Self::generate_code(3)),
        }
    }

    /// Create from danger level (0-3)
    pub fn from_level(level: u8) -> Self {
        match DangerLevel::from_u8(level) {
            DangerLevel::Safe => Self::safe(),
            DangerLevel::Caution => Self::caution(),
            DangerLevel::Dangerous => Self::dangerous(),
            DangerLevel::Critical => Self::critical(),
        }
    }

    /// Check if user input confirms the operation
    pub fn check(&self, input: &str) -> bool {
        let input = input.trim();
        match self.level {
            DangerLevel::Safe => {
                let input = input.to_lowercase();
                input.is_empty() || input == "y" || input == "yes"
            }
            DangerLevel::Caution => input.to_lowercase() == "yes",
            DangerLevel::Dangerous | DangerLevel::Critical => {
                let expected = format!("yes {}", self.code.as_ref().unwrap());
                input == expected
            }
        }
    }

    /// Get the confirmation code if any
    pub fn code(&self) -> Option<&str> {
        self.code.as_deref()
    }

    /// Get the warning message for this level
    pub fn warning(&self) -> Option<&'static str> {
        match self.level {
            DangerLevel::Safe => None,
            DangerLevel::Caution => Some("⚠️  CAUTION: This operation may have side effects."),
            DangerLevel::Dangerous => Some("⚠️  WARNING: Destructive operation detected!"),
            DangerLevel::Critical => Some("🚨 CRITICAL: Extremely dangerous operation!"),
        }
    }

    /// Generate random code for given level
    /// Level 2: 3 chars with at least 1 digit AND 1 lowercase
    /// Level 3: 6 chars with at least 1 digit AND 1 lowercase AND 1 uppercase
    fn generate_code(level: u8) -> String {
        match level {
            2 => Self::generate_mixed_code(3, true, false),
            3 => Self::generate_mixed_code(6, true, true),
            _ => String::new(),
        }
    }

    /// Generate code with guaranteed character types
    /// Level 2: 3 chars with at least 1 digit AND 1 lowercase (digits + lowercase only)
    /// Level 3: 6 chars with at least 1 digit AND 1 lowercase AND 1 uppercase
    fn generate_mixed_code(len: usize, require_lower: bool, require_upper: bool) -> String {
        let digits = "0123456789";
        let lowercase = "abcdefghijklmnopqrstuvwxyz";
        let uppercase = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

        let mut code = String::with_capacity(len);

        // Ensure required character types are present
        code.push(digits.chars().nth(rand::random::<usize>() % digits.len()).unwrap());

        if require_lower {
            code.push(lowercase.chars().nth(rand::random::<usize>() % lowercase.len()).unwrap());
        }

        if require_upper {
            code.push(uppercase.chars().nth(rand::random::<usize>() % uppercase.len()).unwrap());
        }

        // Fill remaining with allowed chars (level 2: digits+lower, level 3: all)
        let allowed_chars = if require_upper {
            // Level 3: digits + lowercase + uppercase
            format!("{}{}{}", digits, lowercase, uppercase)
        } else {
            // Level 2: digits + lowercase only
            format!("{}{}", digits, lowercase)
        };
        let allowed_chars: Vec<char> = allowed_chars.chars().collect();

        while code.len() < len {
            code.push(allowed_chars[rand::random::<usize>() % allowed_chars.len()]);
        }

        // Shuffle to avoid predictable positions
        let mut chars: Vec<char> = code.chars().collect();
        for i in (1..chars.len()).rev() {
            let j = rand::random::<usize>() % (i + 1);
            chars.swap(i, j);
        }

        chars.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_confirmation() {
        let confirm = Confirmation::safe();
        assert!(confirm.check(""));
        assert!(confirm.check("y"));
        assert!(confirm.check("yes"));
        assert!(confirm.check("Y"));
        assert!(confirm.check("YES"));
        assert!(!confirm.check("no"));
    }

    #[test]
    fn test_caution_confirmation() {
        let confirm = Confirmation::caution();
        assert!(confirm.check("yes"));
        assert!(confirm.check("YES"));
        assert!(!confirm.check(""));
        assert!(!confirm.check("y"));
        assert!(!confirm.check("no"));
    }

    #[test]
    fn test_dangerous_confirmation() {
        let confirm = Confirmation::dangerous();
        let code = confirm.code().unwrap();
        assert_eq!(code.len(), 3);

        // Must match exactly "yes <code>"
        assert!(confirm.check(&format!("yes {}", code)));
        assert!(!confirm.check("yes"));
        assert!(!confirm.check("YES"));
        assert!(!confirm.check(""));

        // Code should only contain lowercase and digits
        assert!(code.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));
    }

    #[test]
    fn test_critical_confirmation() {
        let confirm = Confirmation::critical();
        let code = confirm.code().unwrap();
        assert_eq!(code.len(), 6);

        assert!(confirm.check(&format!("yes {}", code)));
        assert!(!confirm.check("yes"));

        // Code should contain uppercase, lowercase, or digits
        assert!(code.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_from_level() {
        assert_eq!(Confirmation::from_level(0).level, DangerLevel::Safe);
        assert_eq!(Confirmation::from_level(1).level, DangerLevel::Caution);
        assert_eq!(Confirmation::from_level(2).level, DangerLevel::Dangerous);
        assert_eq!(Confirmation::from_level(3).level, DangerLevel::Critical);
        assert_eq!(Confirmation::from_level(10).level, DangerLevel::Critical); // clamped
    }

    #[test]
    fn test_danger_level_from_u8() {
        assert_eq!(DangerLevel::from_u8(0), DangerLevel::Safe);
        assert_eq!(DangerLevel::from_u8(1), DangerLevel::Caution);
        assert_eq!(DangerLevel::from_u8(2), DangerLevel::Dangerous);
        assert_eq!(DangerLevel::from_u8(3), DangerLevel::Critical);
        assert_eq!(DangerLevel::from_u8(255), DangerLevel::Critical); // clamped
    }
}
