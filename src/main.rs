use std::env;
use std::process::{self, Command};
use ray::{PackageManager, Winget, command_handler};
use log::{info, error};

/// Ray - A command-line tool that maps pacman commands to winget commands
/// 
/// This tool allows users familiar with pacman syntax to use those same commands
/// on Windows with winget as the underlying package manager.
fn main() {
    // Initialize logger
    env_logger::init();
    
    if cfg!(target_os = "windows") == false {
        error!("This program only runs on Windows.");
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
        match Command::new(package_manager.get_executable_name())
            .arg("--help")
            .status() {
            Ok(status) => process::exit(status.code().unwrap_or(1)),
            Err(e) => {
                eprintln!("Error: Failed to execute {}: {}", package_manager.get_executable_name(), e);
                process::exit(1);
            }
        }
    }

    // Check if the package manager is available
    if !package_manager.is_available() {
        error!("Package manager {} is not available or not installed", package_manager.get_executable_name());
        eprintln!("Error: {} is not available or not installed.", package_manager.get_executable_name());
        process::exit(1);
    }
    
    info!("Ray starting with args: {:?}", args);

    let cmd = &args[0];

    let command_mapping = package_manager.get_command_mapping();
    if let Some(winget_cmds) = command_mapping.get(cmd.as_str()) {
        info!("Mapped command '{}' to: {:?}", cmd, winget_cmds);
        if let Err(e) = command_handler::run_commands(package_manager.get_executable_name(), winget_cmds, &args) {
            error!("Command execution failed: {}", e);
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    } else {
        info!("No mapping found for '{}', running direct command", cmd);
        // Default to package manager command
        if let Err(e) = command_handler::run_direct_command(package_manager.get_executable_name(), &args) {
            error!("Direct command execution failed: {}", e);
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
