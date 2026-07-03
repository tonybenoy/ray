<#
.SYNOPSIS
    Installs ray - a pacman-style command wrapper for winget.

.DESCRIPTION
    Runs entirely in user scope; no administrator rights required. It:
      1. Downloads the release ray.exe from GitHub (latest by default).
      2. Removes the Mark-of-the-Web so SmartScreen stops warning.
      3. (default) Self-signs the binary with a per-user certificate and trusts
         that certificate for the current user, so Smart App Control will run it
         ON THIS MACHINE. The certificate is local-only - it grants no trust to
         anyone else and is not a substitute for real code signing.
      4. Adds the install directory to your user PATH.

.PARAMETER Version
    Release tag to install (e.g. v0.2.0). Defaults to 'latest'.

.PARAMETER InstallDir
    Target directory. Defaults to %USERPROFILE%\.local\bin.

.PARAMETER NoSign
    Skip local self-signing (use if Smart App Control is off on your machine).

.PARAMETER SkipPath
    Do not modify PATH.

.EXAMPLE
    irm https://raw.githubusercontent.com/tonybenoy/ray/main/install.ps1 | iex

.EXAMPLE
    .\install.ps1 -Version v0.2.0
#>
[CmdletBinding()]
param(
    [string]$Version    = 'latest',
    [string]$InstallDir = "$env:USERPROFILE\.local\bin",
    [switch]$NoSign,
    [switch]$SkipPath
)

$ErrorActionPreference = 'Stop'
$Repo = 'tonybenoy/ray'

function Write-Step($msg) { Write-Host "==> $msg" -ForegroundColor Cyan }

# 1. Resolve the download URL.
$url = if ($Version -eq 'latest') {
    "https://github.com/$Repo/releases/latest/download/ray.exe"
} else {
    "https://github.com/$Repo/releases/download/$Version/ray.exe"
}

# 2. Download into place.
$dest = Join-Path $InstallDir 'ray.exe'
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Write-Step "Downloading ray ($Version) -> $dest"
Invoke-WebRequest -Uri $url -OutFile $dest -UseBasicParsing

# 3. Strip Mark-of-the-Web (quiets SmartScreen; does not by itself satisfy SAC).
Unblock-File $dest -ErrorAction SilentlyContinue

# 4. Optionally self-sign so Smart App Control runs it on this machine.
if (-not $NoSign) {
    Write-Step 'Self-signing for local Smart App Control trust'
    $subject = 'CN=ray-local-signing'
    $cert = Get-ChildItem Cert:\CurrentUser\My |
        Where-Object { $_.Subject -eq $subject -and $_.NotAfter -gt (Get-Date) } |
        Select-Object -First 1
    if (-not $cert) {
        $cert = New-SelfSignedCertificate -Type CodeSigningCert -Subject $subject `
            -CertStoreLocation Cert:\CurrentUser\My -KeyUsage DigitalSignature `
            -KeyExportPolicy NonExportable -NotAfter (Get-Date).AddYears(5)
    }
    # Trust the signer for the current user only (no admin, no prompt).
    $store = [System.Security.Cryptography.X509Certificates.X509Store]::new('TrustedPublisher', 'CurrentUser')
    $store.Open('ReadWrite')
    $store.Add($cert)
    $store.Close()
    Set-AuthenticodeSignature -FilePath $dest -Certificate $cert -HashAlgorithm SHA256 | Out-Null
    Write-Host "    signed with $subject"
}

# 5. Ensure the install directory is on the user PATH.
if (-not $SkipPath) {
    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $parts = $userPath -split ';' | Where-Object { $_ }
    if ($parts -notcontains $InstallDir) {
        Write-Step "Adding $InstallDir to user PATH"
        $newPath = ($userPath.TrimEnd(';') + ';' + $InstallDir).TrimStart(';')
        [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
        $env:Path += ";$InstallDir"
    }
}

Write-Step "Installed. Run 'ray -h' (restart your terminal if 'ray' is not found yet)."
