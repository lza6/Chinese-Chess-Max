[CmdletBinding()]
param(
    [Parameter(Mandatory = $false, ValueFromRemainingArguments = $true)]
    [string[]]$Patterns
)
# 用法: powershell -File scripts/ci/check-resources.ps1 "libs/windows-cpu/*.dll" "libs/pikafish/*"
# 全部存在返回 0；任一缺失返回 1（配合 GitHub Actions ::notice:: 输出使用）
$ErrorActionPreference = "Stop"
if (-not $Patterns -or $Patterns.Count -eq 0) {
    Write-Output "用法: check-resources.ps1 <glob> [<glob> ...]"
    exit 2
}
$found = @()
$missing = @()
foreach ($p in $Patterns) {
    $items = Get-ChildItem -Path $p -File -ErrorAction SilentlyContinue
    if ($items) { $found += $p } else { $missing += $p }
}
if ($found.Count -gt 0) { Write-Output "found: $($found -join ', ')" }
if ($missing.Count -gt 0) { Write-Output "missing: $($missing -join ', ')" }
if ($missing.Count -eq 0) { exit 0 } else { exit 1 }