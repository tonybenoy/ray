use std::collections::HashMap;
use std::env;
use std::process::{self, Command};

fn main() {
    if cfg!(target_os = "windows") == false {
        eprintln!("This program only runs on Windows.");
        process::exit(1);
    }
    // Get command line arguments, excluding the program name
    let args: Vec<String> = env::args().skip(1).collect();
    // Check for help flag or empty arguments
    let help_flags = vec!["-h", "--help"];
    if env::args().any(|arg| help_flags.contains(&arg.as_str())) || args.is_empty() {
        println!("Usage: ray [options] [package]");
        println!("Options:");
        println!("  -Syu       Upgrade all packages");
        println!("  -Syyu      Update sources and upgrade all packages");
        println!("  -Sy        Update sources");
        println!("  -S         Install package");
        println!("  -Ss        Search for package");
        println!("  -R         Uninstall package");
        println!("  -Rns       Uninstall package and dependencies");
        println!("  -Q         List installed packages");
        println!("  -Qi        Show package details");
        println!("  -Si        Show package details from remote");
        println!("  -Qs        List installed packages matching search term");
        println!("  -h, --help  Show this help message");
        println!();
        println!("Winget commands:");
        let status = Command::new("winget")
            .arg("--help")
            .status()
            .expect("Failed to execute process");
        process::exit(status.code().unwrap_or(1));
    }

    // Mapping of pacman commands to winget commands
    let mut pacman_to_winget: HashMap<&str, Vec<&str>> = HashMap::new();
    pacman_to_winget.insert("-Syu", vec!["upgrade", "--all"]);
    pacman_to_winget.insert(
        "-Syyu",
        vec!["source", "update", "&&", "winget", "upgrade", "--all"],
    );
    pacman_to_winget.insert("-Sy", vec!["source", "update"]);
    pacman_to_winget.insert("-S", vec!["install"]);
    pacman_to_winget.insert("-Ss", vec!["search"]);
    pacman_to_winget.insert("-R", vec!["uninstall"]);
    pacman_to_winget.insert("-Rns", vec!["uninstall"]);
    pacman_to_winget.insert("-Q", vec!["list"]);
    pacman_to_winget.insert("-Qi", vec!["show"]);
    pacman_to_winget.insert("-Si", vec!["show"]);
    pacman_to_winget.insert("-Qs", vec!["list"]);

    let cmd = &args[0];

    if pacman_to_winget.contains_key(cmd.as_str()) {
        let mut winget_cmd: Vec<String> = vec!["winget".to_string()];
        winget_cmd.extend(
            pacman_to_winget
                .get(cmd.as_str())
                .unwrap()
                .iter()
                .map(|&s| s.to_string()),
        );

        // Commands that require additional arguments
        let requires_arg = vec!["-S", "-Ss", "-R", "-Qi", "-Si", "-Rns"];
        if requires_arg.contains(&cmd.as_str()) {
            if args.len() > 1 {
                winget_cmd.extend(args[1..].to_vec());
            } else {
                println!(
                    "Error: The command '{}' requires a package name or search term.",
                    cmd
                );
                process::exit(1);
            }
        }

        println!("Running: {}", winget_cmd.join(" "));
        let status = Command::new(&winget_cmd[0])
            .args(&winget_cmd[1..])
            .status()
            .expect("Failed to execute process");

        process::exit(status.code().unwrap_or(1));
    } else {
        // Default to winget command
        let mut winget_cmd: Vec<String> = vec!["winget".to_string()];
        winget_cmd.extend(args.iter().cloned());

        println!("Running: {}", winget_cmd.join(" "));
        let status = Command::new(&winget_cmd[0])
            .args(&winget_cmd[1..])
            .status()
            .expect("Failed to execute process");

        process::exit(status.code().unwrap_or(1));
    }
}
