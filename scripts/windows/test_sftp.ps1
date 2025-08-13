# Hinglish: Windows client se quick SFTP smoke test chalayein.
# Steps:
#  - SSH key generate (agar nahi hai)
#  - Friend ki mac pe public key add karni hogi (manual ya scp se)
#  - Known_hosts seed (pehli baar AcceptNew policy use kar sakte ho)
#  - sftp-cli ke sftp ls/upload/download try karo

param(
  [string]$Host = "",
  [int]$Port = 22,
  [string]$User = "",
  [string]$Password = "",
  [string]$KnownHosts = "$env:USERPROFILE\.ssh\known_hosts",
  [string]$KeyPath = "$env:USERPROFILE\.ssh\id_rsa",
  [string]$CliExe = "sftp-cli.exe",
  [string]$TestFile = "test_upload.txt"
)

$ErrorActionPreference = 'Stop'

function Ensure-File($path) {
  $dir = Split-Path -Parent $path
  if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir | Out-Null }
  if (-not (Test-Path $path)) { New-Item -ItemType File -Path $path | Out-Null }
}

# 1) SSH keypair ensure
if (-not (Test-Path $KeyPath)) {
  Write-Host "Generating SSH key at $KeyPath"
  ssh-keygen -t rsa -b 4096 -N "" -f $KeyPath | Out-Null
}

# 2) Print pubkey path (isko mac pe authorized_keys me daalna hoga)
$pub = "$KeyPath.pub"
Write-Host "Public key path (share/copy to mac authorized_keys): $pub"

# 3) Prepare known_hosts file
Ensure-File $KnownHosts

# 4) Prepare test file
if (-not (Test-Path $TestFile)) { "hello-from-windows" | Out-File -Encoding utf8 $TestFile }

if ([string]::IsNullOrEmpty($Host) -or [string]::IsNullOrEmpty($User)) {
  Write-Host "Usage: .\scripts\windows\test_sftp.ps1 -Host <mac-ip> -User <mac-username> [-Password <pwd>] [-Port 22]"
  exit 1
}

# 5) Resolve CLI path (either local cargo build or release artifact)
$cliCandidates = @(
  "$PSScriptRoot\..\..\target\debug\sftp-cli.exe",
  "$PSScriptRoot\..\..\target\release\sftp-cli.exe",
  "$PSScriptRoot\..\..\dist\sftp-cli-windows-x86_64\sftp-cli.exe",
  $CliExe
)
$cli = $cliCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $cli) { throw "sftp-cli.exe not found. Build or set -CliExe." }

# 6) Build base args
$base = @("sftp","--host", $Host, "--port", "$Port", "--username", $User, "--known-hosts", $KnownHosts, "--accept-new")
if ($Password) { $base += @("--password", $Password) } else { $base += @("--key", $KeyPath) }

# 7) Run LS
Write-Host "[LS]"; & $cli @($base + @("ls","."))

# 8) Upload then download
$remote = "upload-$([IO.Path]::GetFileName($TestFile))"
Write-Host "[UPLOAD] $TestFile -> $remote"; & $cli @($base + @("upload", $TestFile, $remote))

$outFile = "downloaded-$([IO.Path]::GetFileName($TestFile))"
if (Test-Path $outFile) { Remove-Item $outFile -Force }
Write-Host "[DOWNLOAD] $remote -> $outFile"; & $cli @($base + @("download", $remote, $outFile))

Write-Host "Done. Verify files on both ends."
