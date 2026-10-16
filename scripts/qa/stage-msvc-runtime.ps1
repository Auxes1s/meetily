# Native C++ dependencies import these libraries; Windows does not guarantee
# that a matching Visual C++ Redistributable is already installed.
$ErrorActionPreference = 'Stop'
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
$installation = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $installation) { throw 'A Visual Studio C++ installation is required' }
$directories = @(Get-ChildItem (Join-Path $installation 'VC/Redist/MSVC/*/x64/Microsoft.VC*.CRT') -Directory | Sort-Object FullName -Descending)
if ($directories.Count -eq 0) { throw 'The x64 Microsoft C++ redistributable directory is missing' }
$redist = $directories[0]
$sourceFiles = @(Get-ChildItem $redist.FullName -Filter '*.dll')
foreach ($name in @('msvcp140.dll', 'vcruntime140.dll', 'vcruntime140_1.dll')) {
  if ($name -notin $sourceFiles.Name) { throw "Required C++ runtime missing from redistributable directory: $name" }
}
$destination = Join-Path $PSScriptRoot '../../frontend/src-tauri/binaries/msvc-runtime'
New-Item -ItemType Directory -Force $destination | Out-Null
$manifest = @()
foreach ($file in $sourceFiles) {
  $signature = Get-AuthenticodeSignature $file.FullName
  if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch 'O=Microsoft Corporation(?:,|$)') {
    throw "C++ runtime must have a valid Microsoft signature: $($file.Name)"
  }
  $hash = (Get-FileHash $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
  $target = Join-Path $destination $file.Name
  Copy-Item $file.FullName $target -Force
  if ((Get-FileHash $target -Algorithm SHA256).Hash.ToLowerInvariant() -ne $hash) { throw 'C++ runtime copy mismatch' }
  $manifest += @{
    name = $file.Name
    sha256 = $hash
    version = $file.VersionInfo.FileVersion
    signer = $signature.SignerCertificate.Subject
  }
}
@{
  source = 'Microsoft Visual Studio redistributable x64 CRT directory'
  deployment = 'Application local; update these DLLs with each preview build'
  toolset = $redist.Parent.Parent.Name
  redistribution_information = 'https://learn.microsoft.com/en-us/cpp/windows/redistributing-visual-cpp-files'
  files = $manifest
} | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $destination 'MSVC-runtime-manifest.json') -Encoding utf8
$manifest | ConvertTo-Json -Depth 3
