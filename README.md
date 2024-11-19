# Ray

Ray is a command-line tool that maps common `pacman` commands to `winget` commands, allowing you to use familiar `pacman` syntax on Windows.

## Features

- Map `pacman` commands to `winget` commands
- Supports various `pacman` commands like `-Syu`, `-Syyu`, `-Sy`, `-S`, `-Ss`, `-R`, `-Rns`, `-Q`, `-Qi`, `-Si`, `-Qs`

## Installation

To build and install Ray, you need to have Rust installed. You can install Rust from [rust-lang.org](https://www.rust-lang.org/).

1. Clone the repository:
    ```sh
    git clone https://github.com/tonybenoy/ray.git
    cd ray
    ```

2. Build the project:
    ```sh
    cargo build --release
    ```

3. The executable will be located in the `target/release` directory.

## Usage

Run the `ray` executable with the desired `pacman` command:

```sh
ray [options] [package]
```

Add ray to your PATH to use it from anywhere.


## Supported Commands

- `-Syu`, `-Syyu`: Update all packages
- `-Sy`: Update package database
- `-S`: Install package(s)
- `-Ss`: Search for packages
- `-R`,`-Rns`: Remove package(s)
- `-Q`: List installed packages
- `-Qi`: Show package information
- `-Si`: Show package information
- `-Qs`: Search for installed packages

All other winget commands can be used as well by simple passing the command as an argument to `ray`.
