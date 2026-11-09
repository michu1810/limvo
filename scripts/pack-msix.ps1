<#
.SYNOPSIS
  Packs the built app into an MSIX package for the Microsoft Store.

.DESCRIPTION
  Run after `npx tauri build --no-bundle`. Lays out limvo.exe, the manifest
  and the logos in src-tauri/target/msix/layout and packs them into
  src-tauri/target/msix/Limvo_<version>_x64.msix with makeappx from the
  Windows SDK.

  The package is not signed: the Store signs what it publishes. For a local
  test, register the layout folder instead (needs Developer Mode):
    Add-AppxPackage -Register src-tauri/target/msix/layout/AppxManifest.xml

  The identity values come from Partner Center > the app > Product identity.
  The defaults only serve local tests.
#>
param(
  [string]$IdentityName = "Limvo.Local",
  [string]$Publisher = "CN=Limvo Local",
  [string]$PublisherDisplayName = "Limvo contributors"
)

$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot -Parent
$tauri = Join-Path $root "src-tauri"
$exe = Join-Path $tauri "target/release/limvo.exe"
$out = Join-Path $tauri "target/msix"
$layout = Join-Path $out "layout"

if (-not (Test-Path $exe)) { throw "build the app first: npx tauri build --no-bundle" }

# The Store requires four parts and keeps the last one for itself, so it is 0.
$version = (Get-Content (Join-Path $tauri "tauri.conf.json") -Raw | ConvertFrom-Json).version
if ($version -notmatch '^\d+\.\d+\.\d+$') { throw "version $version is not major.minor.patch" }
$packageVersion = "$version.0"

$makeappx = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\makeappx.exe" -ErrorAction SilentlyContinue |
  Sort-Object { [version]$_.Directory.Parent.Name } -Descending |
  Select-Object -First 1
if (-not $makeappx) { throw "makeappx.exe not found; install the Windows SDK" }

if (Test-Path $layout) { Remove-Item $layout -Recurse -Force }
New-Item -ItemType Directory -Force (Join-Path $layout "Assets") | Out-Null
Copy-Item $exe $layout
foreach ($logo in "StoreLogo.png", "Square44x44Logo.png", "Square150x150Logo.png") {
  Copy-Item (Join-Path $tauri "icons/$logo") (Join-Path $layout "Assets/$logo")
}

# XML-escape the values, since a publisher name may contain & or quotes.
function Escape([string]$text) { [Security.SecurityElement]::Escape($text) }
$manifest = (Get-Content (Join-Path $tauri "msix/AppxManifest.xml") -Raw).
  Replace("{{IDENTITY_NAME}}", (Escape $IdentityName)).
  Replace("{{PUBLISHER}}", (Escape $Publisher)).
  Replace("{{PUBLISHER_DISPLAY_NAME}}", (Escape $PublisherDisplayName)).
  Replace("{{VERSION}}", $packageVersion)
if ($manifest -match '\{\{') { throw "the manifest still has an unfilled value" }
[IO.File]::WriteAllText((Join-Path $layout "AppxManifest.xml"), $manifest, (New-Object Text.UTF8Encoding $false))

$package = Join-Path $out "Limvo_${version}_x64.msix"
& $makeappx.FullName pack /d $layout /p $package /o /h SHA256
if ($LASTEXITCODE -ne 0) { throw "makeappx failed" }
Write-Host "Packed $package"
