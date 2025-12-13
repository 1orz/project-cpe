#!/bin/sh
#
# usb_switch_demo.sh
# USB Gadget 监控脚本，定期发送状态和 dmesg 到飞书
#
# 用法： ./usb_switch_demo.sh <webhook_url> [间隔秒数]
# 停止： kill 进程 或 Ctrl+C
#

WEBHOOK="$1"
INTERVAL="${2:-6}"

if [ -z "$WEBHOOK" ]; then
  echo "用法: $0 <feishu_webhook_url> [间隔秒数，默认6]"
  exit 1
fi

# JSON 转义函数：处理 \、"、换行、制表符等特殊字符
json_escape() {
  printf '%s' "$1" | sed -e 's/\\/\\\\/g' \
                         -e 's/"/\\"/g' \
                         -e 's/	/\\t/g' | \
                    tr '\n' ' '
}

# 截断字符串，避免超过飞书限制（兼容 BusyBox）
truncate_msg() {
  printf '%s' "$1" | cut -c1-1800
}

echo "=================================="
echo "[USB-MON] USB Gadget 监控已启动"
echo "[USB-MON] PID: $$"
echo "[USB-MON] 间隔: ${INTERVAL}s"
echo "[USB-MON] 停止: kill $$ 或 Ctrl+C"
echo "=================================="

# 清空 dmesg 缓冲区，从干净状态开始
dmesg -c >/dev/null 2>&1

COUNT=0
while true; do
  COUNT=$((COUNT + 1))
  TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
  
  # 读取 USB Gadget 状态
  UDC_VAL=$(cat /sys/kernel/config/usb_gadget/g1/UDC 2>/dev/null || echo "N/A")
  VID_VAL=$(cat /sys/kernel/config/usb_gadget/g1/idVendor 2>/dev/null || echo "N/A")
  PID_VAL=$(cat /sys/kernel/config/usb_gadget/g1/idProduct 2>/dev/null || echo "N/A")
  
  # 读取当前功能配置
  FUNC_LIST=""
  if [ -d "/sys/kernel/config/usb_gadget/g1/configs/c.1" ]; then
    FUNC_LIST=$(ls -1 /sys/kernel/config/usb_gadget/g1/configs/c.1 2>/dev/null | grep -v "^strings$" | grep -v "^MaxPower$" | grep -v "^bmAttributes$" | tr '\n' ',' | sed 's/,$//')
  fi
  [ -z "$FUNC_LIST" ] && FUNC_LIST="none"
  
  # 增量读取 dmesg（读取后自动清空缓冲区）
  DMESG_OUT=$(dmesg -c 2>/dev/null | tail -n 20)
  [ -z "$DMESG_OUT" ] && DMESG_OUT="(no new kernel messages)"
  
  # 转义 dmesg 内容
  DMESG_ESCAPED=$(json_escape "$DMESG_OUT")
  
  # 构建消息（使用换行符使飞书显示更清晰）
  MSG="[USB-MON #${COUNT}] ${TIMESTAMP}
---
UDC: ${UDC_VAL}
VID: ${VID_VAL}
PID: ${PID_VAL}
Functions: ${FUNC_LIST}
---
dmesg:
${DMESG_OUT}"

  # 转义整个消息用于 JSON
  MSG_ESCAPED=$(json_escape "$(truncate_msg "$MSG")")
  
  # 构建 JSON 并发送
  JSON_PAYLOAD="{\"msg_type\":\"text\",\"content\":{\"text\":\"${MSG_ESCAPED}\"}}"
  
  # 发送到飞书，带超时和重试
  RESULT=$(curl -m 10 -s -w "%{http_code}" -X POST "$WEBHOOK" \
    -H "Content-Type: application/json" \
    -d "$JSON_PAYLOAD" \
    -o /dev/null 2>&1)
  
  if [ "$RESULT" = "200" ]; then
    echo "[USB-MON] #$COUNT OK - $TIMESTAMP"
  else
    echo "[USB-MON] #$COUNT FAIL (HTTP $RESULT) - $TIMESTAMP"
    # 失败时等待1秒后重试一次
    sleep 1
    RESULT=$(curl -m 10 -s -w "%{http_code}" -X POST "$WEBHOOK" \
      -H "Content-Type: application/json" \
      -d "$JSON_PAYLOAD" \
      -o /dev/null 2>&1)
    [ "$RESULT" = "200" ] && echo "[USB-MON] #$COUNT RETRY OK" || echo "[USB-MON] #$COUNT RETRY FAIL"
  fi
  
  sleep "$INTERVAL"
done
