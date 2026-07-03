<#
.SYNOPSIS
    Updates an installed ray.exe in place to the latest GitHub release.

.DESCRIPTION
    Invoked by `ray --self-update`. Runs in user scope. It:
      1. Checks the latest GitHub release against the running version.
      2. Downloads the new ray.exe.
      3. Self-signs it with the per-user certificate (so Smart App Control keeps
         running it - release binaries are unsigned).
      4. Swaps it into place. A running exe cannot be overwritten, but it can be
         renamed, so the old binary is moved aside and the new one takes its path.

.PARAMETER TargetPath
    Full path to the ray.exe to replace (passed by ray via current_exe()).

.PARAMETER CurrentVersion
    The version of the running ray (passed by ray via CARGO_PKG_VERSION).

.PARAMETER Repo
    owner/name of the GitHub repository. Defaults to tonybenoy/ray.

.PARAMETER NoSign
    Skip local self-signing (use if Smart App Control is off).
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$TargetPath,
    [Parameter(Mandatory)][string]$CurrentVersion,
    [string]$Repo = 'tonybenoy/ray',
    [switch]$NoSign
)

$ErrorActionPreference = 'Stop'
function Write-Step($m) { Write-Host "==> $m" -ForegroundColor Cyan }

# 1. Find the latest release.
$headers = @{ 'User-Agent' = 'ray-self-update' }
$rel = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -Headers $headers
$latest = $rel.tag_name.TrimStart('v')

if ([version]$latest -le [version]$CurrentVersion) {
    Write-Host "ray is already up to date (v$CurrentVersion)."
    exit 0
}
Write-Step "Updating ray v$CurrentVersion -> v$latest"

# 2. Locate the ray.exe asset for that release.
$asset = $rel.assets | Where-Object { $_.name -eq 'ray.exe' } | Select-Object -First 1
if (-not $asset) { Write-Error "Release v$latest has no ray.exe asset."; exit 1 }

# 3. Download it.
$tmp = Join-Path $env:TEMP "ray-update-$latest.exe"
Write-Step "Downloading $($asset.browser_download_url)"
Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $tmp -UseBasicParsing
Unblock-File $tmp -ErrorAction SilentlyContinue

# 4. Self-sign so Smart App Control keeps running it (same cert as install.ps1).
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
    $store = [System.Security.Cryptography.X509Certificates.X509Store]::new('TrustedPublisher', 'CurrentUser')
    $store.Open('ReadWrite')
    $store.Add($cert)
    $store.Close()
    Set-AuthenticodeSignature -FilePath $tmp -Certificate $cert -HashAlgorithm SHA256 | Out-Null
}

# 5. Swap it in. The running exe can be renamed but not overwritten, so move the
#    old binary aside and move the new one into its place.
Write-Step 'Installing update'
$backup = "$TargetPath.old"
Remove-Item -LiteralPath $backup -Force -ErrorAction SilentlyContinue
Move-Item  -LiteralPath $TargetPath -Destination $backup -Force
Move-Item  -LiteralPath $tmp -Destination $TargetPath -Force
Remove-Item -LiteralPath $backup -Force -ErrorAction SilentlyContinue

Write-Host "ray updated to v$latest. Restart your shell or re-run ray." -ForegroundColor Green
