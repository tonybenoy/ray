use std::collections::HashMap;
use std::env;
use std::process::{self, Command};

trait PackageManager {
    fn get_command_mapping(&self) -> HashMap<&str, Vec<&str>>;
    fn get_help_message(&self) -> &str;
    fn get_executable_name(&self) -> &str;
}

struct Winget;

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

fn main() {
    if cfg!(target_os = "windows") == false {
        eprintln!("This program only runs on Windows.");
        process::exit(1);
    }

    let package_manager: Box<dyn PackageManager> = Box::new(Winget);

    // Get command line arguments, excluding the program name
    let args: Vec<String> = env::args().skip(1).collect();
    // Check for help flag or empty arguments
    let help_flags = vec!["-h", "--help"];
    if env::args().any(|arg| help_flags.contains(&arg.as_str())) || args.is_empty() {
        println!("{}", package_manager.get_help_message());
        println!("{} commands:", package_manager.get_executable_name());
        let status = Command::new(package_manager.get_executable_name())
            .arg("--help")
            .status()
            .expect("Failed to execute process");
        process::exit(status.code().unwrap_or(1));
    }

    let cmd = &args[0];
    println!("Command: {}", cmd);

    let command_mapping = package_manager.get_command_mapping();
    if command_mapping.contains_key(cmd.as_str()) {
        let winget_cmds = command_mapping.get(cmd.as_str()).unwrap();
        run_commands(package_manager.get_executable_name(), winget_cmds, &args);
    } else {
        // Default to package manager command
        let mut winget_cmd: Vec<String> = vec![package_manager.get_executable_name().to_string()];
        winget_cmd.extend(args.iter().cloned());

        println!("Running: {}", winget_cmd.join(" "));
        let status = Command::new(&winget_cmd[0])
            .args(&winget_cmd[1..])
            .status()
            .expect("Failed to execute process");

        process::exit(status.code().unwrap_or(1));
    }
}

fn run_commands(executable: &str, commands: &[&str], args: &[String]) {
    let mut i = 0;
    while i < commands.len() {
        let mut winget_cmd: Vec<String> = vec![executable.to_string()];
        while i < commands.len() && commands[i] != "&&" {
            winget_cmd.push(commands[i].to_string());
            i += 1;
        }
        i += 1; // Skip "&&"

        // Commands that require additional arguments
        let requires_arg = vec!["install", "search", "uninstall", "show"];
        if requires_arg.contains(&winget_cmd[1].as_str()) {
            if args.len() > 1 {
                winget_cmd.extend(args[1..].to_vec());
            } else {
                println!(
                    "Error: The command '{}' requires a package name or search term.",
                    winget_cmd[1]
                );
                process::exit(1);
            }
        }

        println!("Running: {}", winget_cmd.join(" "));
        let status = Command::new(&winget_cmd[0])
            .args(&winget_cmd[1..])
            .status()
            .expect("Failed to execute process");

        if !status.success() {
            process::exit(status.code().unwrap_or(1));
        }
    }
}
