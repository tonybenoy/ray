use std::collections::HashMap;
use std::env;
use std::process::{self, Command};

fn main() {
    // Get command line arguments, excluding the program name
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        println!("Usage: ray [options] [package]");
        process::exit(1);
    }

    // Mapping of pacman commands to winget commands
    let mut pacman_to_winget: HashMap<&str, Vec<&str>> = HashMap::new();
    pacman_to_winget.insert("-Syu", vec!["upgrade", "--all"]);
    pacman_to_winget.insert("-Syyu", vec!["upgrade", "--all"]);
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
        let requires_arg = vec!["-S", "-Ss", "-R", "-Qi", "-Si", "-Qs", "-Syyu", "-Rns"];
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
