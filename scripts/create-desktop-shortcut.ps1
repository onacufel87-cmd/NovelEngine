# 在已安装目录或便携 exe 旁创建桌面快捷方式
param(
    [string]$ExePath = ""
)

$ErrorActionPreference = "Stop"

if (-not $ExePath) {
    $releaseExe = Join-Path $PSScriptRoot "..\src-tauri\target\release\novel-reader-core.exe"
    if (Test-Path $releaseExe) {
        $ExePath = (Resolve-Path $releaseExe).Path
    } else {
        Write-Error "未找到 exe。请传入 -ExePath，或先运行「一键打包.bat」。"
    }
}

$ExePath = (Resolve-Path $ExePath).Path
$ShortcutName = "小说引擎.lnk"
$Desktop = [Environment]::GetFolderPath("Desktop")
$ShortcutPath = Join-Path $Desktop $ShortcutName

$Wsh = New-Object -ComObject WScript.Shell
$Link = $Wsh.CreateShortcut($ShortcutPath)
$Link.TargetPath = $ExePath
$Link.WorkingDirectory = Split-Path $ExePath -Parent
$Link.Description = "小说引擎 — 通用书源阅读器"
$Link.Save()

Write-Host "已创建桌面快捷方式: $ShortcutPath"
