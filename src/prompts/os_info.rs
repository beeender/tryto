use std::env;
#[cfg(target_os = "macos")]
use std::process::{Command, Stdio};

/// Platform information for command generation.
#[derive(Debug, Clone)]
struct OsInfo {
    os: &'static str,
    shell: String,
    tool_flavor: String,
}

/// Get the current shell from environment.
fn detect_shell() -> String {
    env::var("SHELL")
        .ok()
        .and_then(|s| s.split('/').next_back().map(String::from))
        .unwrap_or_else(|| "sh".to_string())
}

/// Check if GNU sed is in PATH by testing GNU-specific flag.
#[cfg(target_os = "macos")]
fn is_gnu_tools() -> bool {
    // GNU sed supports --version, BSD sed does not
    Command::new("sed")
        .args(["--version"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Detect the current operating system and tool flavor.
fn detect() -> OsInfo {
    let shell = detect_shell();

    #[cfg(target_os = "macos")]
    {
        let tool_flavor = if is_gnu_tools() {
            "GNU (Homebrew)".to_string()
        } else {
            "BSD (macOS default)".to_string()
        };
        OsInfo {
            os: "macOS",
            shell,
            tool_flavor,
        }
    }
    #[cfg(target_os = "linux")]
    {
        OsInfo {
            os: "Linux",
            shell,
            tool_flavor: "GNU".to_string(),
        }
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        OsInfo {
            os: "Unix-like",
            shell,
            tool_flavor: "POSIX".to_string(),
        }
    }
}

/// Build platform context section for the system prompt.
pub fn build_context() -> String {
    let info = detect();
    format!(
        "## Platform Context\n\n\
        Operating System: {}\n\
        Shell: {}\n\
        Tool Flavor: {}\n\n\
        Important: Generate commands compatible with this platform.\n",
        info.os,
        info.shell,
        info.tool_flavor
    )
}
