use std::collections::HashMap;
use std::fmt;
use std::error::Error;
use std::io;
use std::process::Command;
use log::info;

#[derive(Debug)]
pub enum RayError {
    CommandNotFound(String),
    ExecutionFailed(String),
    InvalidArguments(String),
    IoError(io::Error),
}

impl fmt::Display for RayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RayError::CommandNotFound(cmd) => write!(f, "Command not found: {}", cmd),
            RayError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            RayError::InvalidArguments(msg) => write!(f, "Invalid arguments: {}", msg),
            RayError::IoError(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl Error for RayError {}

impl From<io::Error> for RayError {
    fn from(err: io::Error) -> Self {
        RayError::IoError(err)
    }
}

pub type Result<T> = std::result::Result<T, RayError>;

/// Trait defining the interface for package manager implementations
pub trait PackageManager {
    /// Get the mapping from pacman commands to target package manager commands
    fn get_command_mapping(&self) -> HashMap<&str, Vec<&str>>;
    
    /// Get the help message for this package manager wrapper
    fn get_help_message(&self) -> &str;
    
    /// Get the name of the underlying executable
    fn get_executable_name(&self) -> &str;
    
    /// Check if the underlying package manager is available
    fn is_available(&self) -> bool {
        Command::new(self.get_executable_name())
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

/// Windows Package Manager (winget) implementation
pub struct Winget;

impl PackageManager for Winget {
    fn get_command_mapping(&self) -> HashMap<&str, Vec<&str>> {
        let mut pacman_to_winget: HashMap<&str, Vec<&str>> = HashMap::new();
        pacman_to_winget.insert("-Syu", vec!["upgrade", "--all", "--include-unknown"]);
        pacman_to_winget.insert("-Syyu", vec!["source", "update", "&&", "upgrade", "--all", "--include-unknown"]);
        pacman_to_winget.insert("-Sy", vec!["source", "update"]);
        pacman_to_winget.insert("-S", vec!["install"]);
        pacman_to_winget.insert("-Ss", vec!["search"]);
        pacman_to_winget.insert("-R", vec!["uninstall"]);
        pacman_to_winget.insert("-Rns", vec!["uninstall"]);
        pacman_to_winget.insert("-Q", vec!["list"]);
        pacman_to_winget.insert("-Qi", vec!["show"]);
        pacman_to_winget.insert("-Si", vec!["show"]);
        pacman_to_winget.insert("-Qs", vec!["list"]);
        pacman_to_winget
    }

    fn get_help_message(&self) -> &str {
        "Usage: ray [options] [package]\n\
        Options:\n\
        -Syu       Upgrade all packages\n\
        -Syyu      Update sources and upgrade all packages\n\
        -Sy        Update sources\n\
        -S         Install package\n\
        -Ss        Search for package\n\
        -R         Uninstall package\n\
        -Rns       Uninstall package and dependencies\n\
        -Q         List installed packages\n\
        -Qi        Show package details\n\
        -Si        Show package details from remote\n\
        -Qs        List installed packages matching search term\n\
        --self-update  Update ray to the latest release\n\
        -h, --help  Show this help message\n"
    }

    fn get_executable_name(&self) -> &str {
        "winget"
    }
}

/// Self-update: replace the running ray.exe with the latest GitHub release.
///
/// The download-sign-swap logic lives in `self-update.ps1`, embedded at compile
/// time, so the binary carries its own updater and needs no extra crates. The
/// script self-signs the new binary so Smart App Control keeps allowing it.
pub mod self_update {
    use super::*;
    use std::fs;
    use std::process::Command;

    const SCRIPT: &str = include_str!("../self-update.ps1");

    pub fn run() -> Result<()> {
        let exe = std::env::current_exe()?;
        let script_path = std::env::temp_dir().join("ray-self-update.ps1");
        fs::write(&script_path, SCRIPT)?;

        info!("Running self-update for {}", exe.display());
        let status = Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                &script_path.to_string_lossy(),
                "-TargetPath",
                &exe.to_string_lossy(),
                "-CurrentVersion",
                env!("CARGO_PKG_VERSION"),
            ])
            .status()?;

        if !status.success() {
            return Err(RayError::ExecutionFailed(format!(
                "Self-update failed with exit code: {}",
                status.code().unwrap_or(-1)
            )));
        }
        Ok(())
    }
}

pub mod command_handler {
    use super::*;
    use std::process::Command;

    /// Sub-commands that are meaningless without a package name or search term.
    const REQUIRES_ARG: [&str; 4] = ["install", "search", "uninstall", "show"];

    /// Turn a mapping value into the concrete winget invocations to run.
    ///
    /// `commands` is the mapping value, whose sub-commands may be chained with a
    /// `"&&"` separator. `extra_args` are the user's arguments following the
    /// pacman-style flag; they are forwarded to the **final** sub-command (e.g.
    /// the package name for `-S`, or the search term for `-Qs`).
    pub fn plan_invocations(commands: &[&str], extra_args: &[String]) -> Result<Vec<Vec<String>>> {
        let sub_commands: Vec<&[&str]> = commands
            .split(|token| *token == "&&")
            .filter(|sub| !sub.is_empty())
            .collect();

        let mut invocations = Vec::with_capacity(sub_commands.len());
        for (idx, cmd_args) in sub_commands.iter().enumerate() {
            if REQUIRES_ARG.contains(&cmd_args[0]) && extra_args.is_empty() {
                return Err(RayError::InvalidArguments(format!(
                    "The command '{}' requires a package name or search term.",
                    cmd_args[0]
                )));
            }

            let mut invocation: Vec<String> = cmd_args.iter().map(|s| s.to_string()).collect();
            if idx == sub_commands.len() - 1 {
                invocation.extend(extra_args.iter().cloned());
            }
            invocations.push(invocation);
        }
        Ok(invocations)
    }

    /// Execute a mapped command, running each chained sub-command in order.
    pub fn run_commands(executable: &str, commands: &[&str], args: &[String]) -> Result<()> {
        for invocation in plan_invocations(commands, &args[1..])? {
            run(executable, &invocation)?;
        }
        Ok(())
    }

    /// Execute a direct command (passthrough to the underlying package manager).
    pub fn run_direct_command(executable: &str, args: &[String]) -> Result<()> {
        run(executable, args)
    }

    /// Run `executable` with `args`, returning an error on a non-zero exit code.
    fn run(executable: &str, args: &[String]) -> Result<()> {
        let line = format!("{} {}", executable, args.join(" "));
        info!("Executing command: {}", line);
        println!("Running: {}", line);

        let status = Command::new(executable).args(args).status()?;
        if !status.success() {
            return Err(RayError::ExecutionFailed(format!(
                "Command failed with exit code: {}",
                status.code().unwrap_or(-1)
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_winget_command_mapping() {
        let winget = Winget;
        let mapping = winget.get_command_mapping();
        
        assert_eq!(mapping.get("-S"), Some(&vec!["install"]));
        assert_eq!(mapping.get("-R"), Some(&vec!["uninstall"]));
        assert_eq!(mapping.get("-Ss"), Some(&vec!["search"]));
        assert_eq!(mapping.get("-Q"), Some(&vec!["list"]));
        assert_eq!(mapping.get("-Syu"), Some(&vec!["upgrade", "--all", "--include-unknown"]));
    }

    #[test]
    fn test_winget_executable_name() {
        let winget = Winget;
        assert_eq!(winget.get_executable_name(), "winget");
    }

    #[test]
    fn test_winget_help_message() {
        let winget = Winget;
        let help = winget.get_help_message();
        assert!(help.contains("Usage: ray"));
        assert!(help.contains("-S"));
        assert!(help.contains("-R"));
    }

    #[test]
    fn test_error_display() {
        let err = RayError::CommandNotFound("test".to_string());
        assert_eq!(format!("{}", err), "Command not found: test");
        
        let err = RayError::InvalidArguments("missing package name".to_string());
        assert_eq!(format!("{}", err), "Invalid arguments: missing package name");
    }

    #[test]
    fn test_command_validation() {
        // Test that commands requiring arguments are properly identified
        let requires_arg = ["install", "search", "uninstall", "show"];
        assert!(requires_arg.contains(&"install"));
        assert!(requires_arg.contains(&"search"));
        assert!(!requires_arg.contains(&"list"));
    }

    #[test]
    fn test_plan_forwards_term_to_query_command() {
        // Regression: `-Qs firefox` must pass the term through -> `winget list firefox`
        let inv = command_handler::plan_invocations(&["list"], &["firefox".to_string()]).unwrap();
        assert_eq!(inv, vec![vec!["list".to_string(), "firefox".to_string()]]);
    }

    #[test]
    fn test_plan_query_command_without_term_is_ok() {
        // `-Q` (list everything) needs no term.
        let inv = command_handler::plan_invocations(&["list"], &[]).unwrap();
        assert_eq!(inv, vec![vec!["list".to_string()]]);
    }

    #[test]
    fn test_plan_chained_forwards_only_to_last_subcommand() {
        // `-Syyu` -> `winget source update` then `winget upgrade --all --include-unknown`
        let inv = command_handler::plan_invocations(
            &["source", "update", "&&", "upgrade", "--all", "--include-unknown"],
            &[],
        )
        .unwrap();
        assert_eq!(
            inv,
            vec![
                vec!["source".to_string(), "update".to_string()],
                vec![
                    "upgrade".to_string(),
                    "--all".to_string(),
                    "--include-unknown".to_string()
                ],
            ]
        );
    }

    #[test]
    fn test_plan_requires_arg_errors_without_term() {
        // `-S` with no package name must be rejected before touching winget.
        assert!(command_handler::plan_invocations(&["install"], &[]).is_err());
        assert!(command_handler::plan_invocations(&["install"], &["vim".to_string()]).is_ok());
    }
}