fn main() {
    // 图标变更时强制 tauri-build 重新嵌入 Windows .exe / macOS 资源
    for icon in [
        "icons/icon.ico",
        "icons/icon.icns",
        "icons/icon.png",
        "icons/32x32.png",
        "icons/64x64.png",
        "icons/128x128.png",
        "icons/128x128@2x.png",
    ] {
        println!("cargo:rerun-if-changed={icon}");
    }
    tauri_build::build()
}
