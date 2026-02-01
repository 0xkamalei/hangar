#!/usr/bin/osascript

-- AppleScript 自动化测试脚本
-- 用于测试代理订阅管理器应用

on run
    -- 设置应用路径
    set appPath to ((path to me as text) & "::") as alias
    set projectRoot to (POSIX path of appPath) & "../../"
    set appBundle to projectRoot & "src-tauri/target/release/bundle/macos/proxy-sub-manager.app"
    
    log "🧪 开始自动化测试"
    log "应用路径: " & appBundle
    
    -- 检查应用是否存在
    try
        tell application "System Events"
            if not (exists file appBundle) then
                display dialog "错误：找不到应用包" & return & appBundle buttons {"确定"} default button 1 with icon stop
                return
            end if
        end tell
    on error errMsg
        display dialog "检查应用失败：" & errMsg buttons {"确定"} default button 1 with icon stop
        return
    end try
    
    log "✓ 应用包存在"
    
    -- 启动应用
    try
        log "→ 启动应用..."
        tell application appBundle
            activate
        end tell
        delay 3
        log "✓ 应用已启动"
    on error errMsg
        display dialog "启动应用失败：" & errMsg buttons {"确定"} default button 1 with icon stop
        return
    end try
    
    -- 检查应用是否在运行
    try
        tell application "System Events"
            set appName to name of file appBundle
            set isRunning to exists (process appName)
            
            if isRunning then
                log "✓ 应用正在运行"
                
                -- 等待应用完全加载
                delay 2
                
                -- 显示成功消息
                display dialog "✅ 应用测试成功！" & return & return & "应用已启动并运行正常。" & return & return & "您可以：" & return & "1. 添加订阅" & return & "2. 启动服务器" & return & "3. 使用订阅链接" buttons {"关闭应用", "继续使用"} default button 2 with icon note
                
                if button returned of result is "关闭应用" then
                    tell process appName
                        click menu item "退出" of menu "proxy-sub-manager" of menu bar 1
                    end tell
                    log "→ 应用已关闭"
                end if
            else
                display dialog "❌ 应用未能正常运行" buttons {"确定"} default button 1 with icon caution
            end if
        end tell
    on error errMsg
        display dialog "检查应用状态失败：" & errMsg buttons {"确定"} default button 1 with icon stop
    end try
    
    log "🎉 测试完成"
end run
