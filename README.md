# Ray

Ray is a command-line tool that maps common `pacman` commands to `winget` commands, allowing you to use familiar `pacman` syntax on Windows.

## Features

- Map `pacman` commands to `winget` commands
- Supports various `pacman` commands like `-Syu`, `-Syyu`, `-Sy`, `-S`, `-Ss`, `-R`, `-Rns`, `-Q`, `-Qi`, `-Si`, `-Qs`

## Installation

### Build from Source

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

### Install script (recommended)

Run this in PowerShell. It downloads the latest release into `%USERPROFILE%\.local\bin`,
adds that directory to your PATH, and works entirely in user scope (no admin required):

```powershell
irm https://raw.githubusercontent.com/tonybenoy/ray/main/install.ps1 | iex
```

By default the script also **self-signs** the binary with a per-user certificate so it
runs under [Smart App Control](https://support.microsoft.com/windows/smart-app-control).
This is required because release binaries are currently unsigned, and Smart App Control
(on by default on many clean Windows 11 installs) blocks unsigned executables. The
certificate is local to your machine and grants no trust to anyone else.

Options:

```powershell
# Pin a version, or skip signing if Smart App Control is off on your machine:
.\install.ps1 -Version v0.2.0
.\install.ps1 -NoSign
```

> Note: because the certificate is self-signed and local, the exe still shows as
> "unknown publisher". Proper code signing of releases is planned.

### Install from github releases (manual)

Download the latest release from the [releases page](https://github.com/tonybenoy/ray/releases/latest)
and add the executable to your PATH. If Smart App Control blocks it, either use the install
script above (which self-signs it for you) or run the tool from a build you compiled yourself.

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

## Updating

Update ray to the latest release in place:

```sh
ray --self-update
```

This downloads the newest release, re-signs it locally (so Smart App Control keeps
allowing it — see [Installation](#installation)), and swaps it into place. If you are
already on the latest version it does nothing.

