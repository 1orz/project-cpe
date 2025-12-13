#!/bin/bash
###
 # @Author: 1orz cloudorzi@gmail.com
 # @Date: 2025-12-07 07:33:11
 # @LastEditors: 1orz cloudorzi@gmail.com
 # @LastEditTime: 2025-12-13 12:51:09
 # @FilePath: /udx710-backend/scripts/monitor.sh
 # @Description: 
 # 
 # Copyright (c) 2025 by 1orz, All Rights Reserved. 
### 
# ofono D-Bus 监控脚本
# 在目标设备上运行

echo "📡 ofono D-Bus 监控工具"
echo ""
echo "选择监控模式:"
echo "  1) 监听 ofono 发出的信号"
echo "  2) 监听发给 ofono 的调用"
echo "  3) 监听短信信号"
echo "  4) 监听通话信号"
echo "  5) 监听所有 ofono 消息"
echo "  6) ofono 调试模式 (需要 root)"
echo ""

read -p "请选择 [1-6]: " choice

case $choice in
    1)
        echo "监听 ofono 发出的信号..."
        dbus-monitor --system "sender='org.ofono'"
        ;;
    2)
        echo "监听发给 ofono 的调用..."
        dbus-monitor --system "destination='org.ofono'"
        ;;
    3)
        echo "监听短信信号..."
        dbus-monitor --system "interface='org.ofono.MessageManager'"
        ;;
    4)
        echo "监听通话信号..."
        dbus-monitor --system "interface='org.ofono.VoiceCallManager'"
        ;;
    5)
        echo "监听所有 ofono 消息..."
        dbus-monitor --system "sender='org.ofono'" &
        dbus-monitor --system "destination='org.ofono'"
        ;;
    6)
        echo "启动 ofono 调试模式..."
        echo "注意: 会先停止当前 ofono 服务"
        read -p "继续? [y/N]: " confirm
        if [ "$confirm" = "y" ] || [ "$confirm" = "Y" ]; then
            systemctl stop ofono 2>/dev/null || true
            /usr/sbin/ofonod -n -d
        fi
        ;;
    *)
        echo "无效选择"
        exit 1
        ;;
esac

