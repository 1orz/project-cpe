#!/bin/bash
###
 # @Author: 1orz cloudorzi@gmail.com
 # @Date: 2025-12-09 17:34:01
 # @LastEditors: 1orz cloudorzi@gmail.com
 # @LastEditTime: 2025-12-13 12:51:13
 # @FilePath: /udx710-backend/scripts/pack-userdata.sh
 # @Description: 
 # 
 # Copyright (c) 2025 by 1orz, All Rights Reserved. 
### 
# 打包 userdata UBIFS 镜像脚本
# 
# 使用方法:
#   ./scripts/pack-userdata.sh [选项]
#
# 依赖工具:
#   - mtd-utils (mkfs.ubifs)
#   - 在 macOS 上需要通过 Docker 或 Linux 虚拟机执行
#
# 安装依赖 (Ubuntu/Debian):
#   sudo apt-get install mtd-utils

set -e

# 切换到项目根目录
cd "$(dirname "$0")/.."

# ==================== 配置参数 ====================
# 这些参数需要根据你的设备 NAND Flash 规格调整
# 可以通过 ubireader_display_info 或 ubinfo 获取原始镜像参数

# UDX710 userdata 分区参数 (从原始镜像获取)
MIN_IO_SIZE=${MIN_IO_SIZE:-2048}       # -m: 最小 I/O 单元大小 (NAND 页大小)
LEB_SIZE=${LEB_SIZE:-126976}           # -e: 逻辑擦除块大小
MAX_LEB_CNT=${MAX_LEB_CNT:-516}        # -c: 最大逻辑擦除块数量

# 输入输出路径
USERDATA_DIR="userdata"
OUTPUT_FILE="userdata.bin"

# ==================== 解析命令行参数 ====================
SKIP_COPY=false
SHOW_INFO=false

for arg in "$@"; do
    case $arg in
        --skip-copy)
            SKIP_COPY=true
            ;;
        --info)
            SHOW_INFO=true
            ;;
        --output=*)
            OUTPUT_FILE="${arg#*=}"
            ;;
        --min-io=*)
            MIN_IO_SIZE="${arg#*=}"
            ;;
        --leb-size=*)
            LEB_SIZE="${arg#*=}"
            ;;
        --max-leb=*)
            MAX_LEB_CNT="${arg#*=}"
            ;;
        --help|-h)
            echo "用法: ./scripts/pack-userdata.sh [选项]"
            echo ""
            echo "选项:"
            echo "  --skip-copy       跳过复制构建产物到 userdata"
            echo "  --info            显示 UBIFS 参数信息后退出"
            echo "  --output=FILE     指定输出文件名 (默认: img-0_vol-userdata.ubifs)"
            echo "  --min-io=SIZE     最小 I/O 单元大小 (默认: $MIN_IO_SIZE)"
            echo "  --leb-size=SIZE   逻辑擦除块大小 (默认: $LEB_SIZE)"
            echo "  --max-leb=COUNT   最大逻辑擦除块数量 (默认: $MAX_LEB_CNT)"
            echo "  --help, -h        显示帮助信息"
            echo ""
            echo "环境变量:"
            echo "  MIN_IO_SIZE       最小 I/O 单元大小"
            echo "  LEB_SIZE          逻辑擦除块大小"
            echo "  MAX_LEB_CNT       最大逻辑擦除块数量"
            echo ""
            echo "获取参数方法:"
            echo "  1. 使用 ubireader_display_info 查看原始镜像"
            echo "  2. 在设备上执行 ubinfo /dev/ubi0 查看"
            exit 0
            ;;
    esac
done

# 显示参数信息
if [ "$SHOW_INFO" = true ]; then
    echo "UBIFS 打包参数:"
    echo "  最小 I/O 单元大小 (-m): $MIN_IO_SIZE"
    echo "  逻辑擦除块大小 (-e):    $LEB_SIZE"
    echo "  最大逻辑擦除块数 (-c):  $MAX_LEB_CNT"
    echo ""
    echo "输出文件: $OUTPUT_FILE"
    exit 0
fi

# ==================== 检查工具 ====================
echo "========================================"
echo "  UBIFS 打包工具"
echo "========================================"
echo ""

if ! command -v mkfs.ubifs &> /dev/null; then
    echo "mkfs.ubifs 不可用，尝试使用 Docker..."
    echo ""
    
    if command -v docker &> /dev/null; then
        # macOS/Windows: 自动使用 Docker 执行
        DOCKER_ARGS="--skip-copy"
        [ "$SKIP_COPY" = false ] && DOCKER_ARGS=""
        
        docker run --rm -v "$(pwd)":/work -w /work ubuntu:22.04 bash -c \
            "apt-get update -qq && apt-get install -y -qq mtd-utils >/dev/null 2>&1 && ./scripts/pack-userdata.sh $DOCKER_ARGS"
        exit $?
    else
        echo "错误: mkfs.ubifs 和 Docker 都不可用"
        echo ""
        echo "安装方法:"
        echo "  Ubuntu/Debian: sudo apt-get install mtd-utils"
        echo "  Arch Linux:    sudo pacman -S mtd-utils"
        echo "  macOS/Windows: brew install --cask docker"
        exit 1
    fi
fi

# ==================== 检查 userdata 目录 ====================
if [ ! -d "$USERDATA_DIR" ]; then
    echo "错误: userdata 目录不存在"
    echo "请先提取 UBIFS: ubireader_extract_files -o userdata 'img-0_vol-userdata.ubifs'"
    exit 1
fi

# ==================== 复制构建产物 ====================
if [ "$SKIP_COPY" = false ]; then
    echo "复制构建产物到 userdata..."
    echo ""
    
    BACKEND_BIN="backend/target/aarch64-unknown-linux-gnu/release/udx710"
    FRONTEND_DIR="frontend/dist"
    TARGET_ROOT="$USERDATA_DIR/home/root"
    TARGET_WWW="$TARGET_ROOT/www"
    
    # 检查并复制后端二进制
    if [ -f "$BACKEND_BIN" ]; then
        echo "  复制后端: $BACKEND_BIN -> $TARGET_ROOT/udx710"
        cp "$BACKEND_BIN" "$TARGET_ROOT/udx710"
        chmod 755 "$TARGET_ROOT/udx710"
    else
        echo "  警告: 后端二进制不存在 ($BACKEND_BIN)"
        echo "        请先运行 ./scripts/build.sh --backend-only"
    fi
    
    # 检查并复制前端文件
    if [ -d "$FRONTEND_DIR" ]; then
        echo "  复制前端: $FRONTEND_DIR -> $TARGET_WWW"
        mkdir -p "$TARGET_WWW"
        rm -rf "$TARGET_WWW"/*
        cp -r "$FRONTEND_DIR"/* "$TARGET_WWW/"
        # 设置前端文件权限
        find "$TARGET_WWW" -type f -exec chmod 644 {} \;
        find "$TARGET_WWW" -type d -exec chmod 755 {} \;
    else
        echo "  警告: 前端构建产物不存在 ($FRONTEND_DIR)"
        echo "        请先运行 ./scripts/build.sh --frontend-only"
    fi
    
    echo ""
fi

# ==================== 设置文件权限 ====================
# 确保所有脚本文件有可执行权限（在 Docker 容器中特别重要）
echo "设置文件权限..."
TARGET_ROOT="$USERDATA_DIR/home/root"

# 设置脚本和二进制文件的可执行权限
chmod 755 "$TARGET_ROOT/loader.sh" 2>/dev/null || true
chmod 755 "$TARGET_ROOT/udx710" 2>/dev/null || true
chmod 755 "$TARGET_ROOT/ttyd/start.sh" 2>/dev/null || true
chmod 755 "$TARGET_ROOT/ttyd/ttyd" 2>/dev/null || true
chmod 755 "$TARGET_ROOT/busybox-aarch64" 2>/dev/null || true

# 设置目录权限
find "$USERDATA_DIR" -type d -exec chmod 755 {} \; 2>/dev/null || true

echo ""

# ==================== 创建 UBIFS 镜像 ====================
echo "创建 UBIFS 镜像..."
echo "  输入目录: $USERDATA_DIR"
echo "  输出文件: $OUTPUT_FILE"
echo "  参数: -m $MIN_IO_SIZE -e $LEB_SIZE -c $MAX_LEB_CNT"
echo ""

# 执行 mkfs.ubifs
mkfs.ubifs \
    -r "$USERDATA_DIR" \
    -m "$MIN_IO_SIZE" \
    -e "$LEB_SIZE" \
    -c "$MAX_LEB_CNT" \
    -o "$OUTPUT_FILE"

echo ""
echo "========================================"
echo "  打包完成!"
echo "========================================"
echo ""
echo "输出文件: $OUTPUT_FILE"
ls -lh "$OUTPUT_FILE"
echo ""
echo "刷写到设备:"
echo "  1. 进入 fastboot 模式"
echo "  2. fastboot flash userdata $OUTPUT_FILE"
echo ""

