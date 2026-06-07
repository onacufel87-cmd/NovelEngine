; NSIS 安装钩子：补全桌面快捷方式，并在卸载时清理
; Tauri 默认已创建「开始菜单」快捷方式；此处确保安装完成后桌面也有入口

!macro NSIS_HOOK_POSTINSTALL
  CreateShortcut "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  Delete "$DESKTOP\${PRODUCTNAME}.lnk"
!macroend
