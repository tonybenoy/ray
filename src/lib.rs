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
        pacman_to_winget.insert("-Syu", vec!["upgrade", "--all"]);
        pacman_to_winget.insert("-Syyu", vec!["source", "update", "&&", "upgrade", "--all"]);
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
        -h, --help  Show this help message\n"
    }

    fn get_executable_name(&self) -> &str {
        "winget"
    }
}

pub mod command_handler {
    use super::*;
    use std::process::Command;

    /// Execute a sequence of commands, handling chained commands separated by "&&"
    pub fn run_commands(executable: &str, commands: &[&str], args: &[String]) -> Result<()> {
        let mut i = 0;
        while i < commands.len() {
            let mut cmd_args = Vec::new();
            while i < commands.len() && commands[i] != "&&" {
                cmd_args.push(commands[i]);
                i += 1;
            }
            i += 1; // Skip "&&"

            if cmd_args.is_empty() {
                continue;
            }

            // Commands that require additional arguments
            let requires_arg = ["install", "search", "uninstall", "show"];
            if requires_arg.contains(&cmd_args[0]) {
                if args.len() <= 1 {
                    return Err(RayError::InvalidArguments(
                        format!("The command '{}' requires a package name or search term.", cmd_args[0])
                    ));
                }
            }

            execute_single_command(executable, &cmd_args, args)?;
        }
        Ok(())
    }

    /// Execute a single command with proper error handling
    fn execute_single_command(executable: &str, cmd_args: &[&str], user_args: &[String]) -> Result<()> {
        let mut full_cmd = Vec::with_capacity(cmd_args.len() + user_args.len() + 1);
        full_cmd.push(executable);
        full_cmd.extend(cmd_args);
        
        // Add user arguments if this command needs them
        let requires_arg = ["install", "search", "uninstall", "show"];
        if !cmd_args.is_empty() && requires_arg.contains(&cmd_args[0]) && user_args.len() > 1 {
            for arg in &user_args[1..] {
                full_cmd.push(arg.as_str());
            }
        }

        info!("Executing command: {}", full_cmd.join(" "));
        println!("Running: {}", full_cmd.join(" "));
        
        let status = Command::new(executable)
            .args(&full_cmd[1..])
            .status()?;

        if !status.success() {
            return Err(RayError::ExecutionFailed(
                format!("Command failed with exit code: {}", status.code().unwrap_or(-1))
            ));
        }
        
        Ok(())
    }

    /// Execute a direct command (passthrough to the underlying package manager)
    pub fn run_direct_command(executable: &str, args: &[String]) -> Result<()> {
        let mut cmd_parts = Vec::with_capacity(args.len() + 1);
        cmd_parts.push(executable);
        for arg in args {
            cmd_parts.push(arg.as_str());
        }
        info!("Executing direct command: {}", cmd_parts.join(" "));
        println!("Running: {}", cmd_parts.join(" "));
        
        let status = Command::new(executable)
            .args(args)
            .status()?;

        if !status.success() {
            return Err(RayError::ExecutionFailed(
                format!("Command failed with exit code: {}", status.code().unwrap_or(-1))
            ));
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
        assert_eq!(mapping.get("-Syu"), Some(&vec!["upgrade", "--all"]));
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
}