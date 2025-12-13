#!/bin/bash
###
 # @Author: 1orz cloudorzi@gmail.com
 # @Date: 2025-12-11 17:44:29
 # @LastEditors: 1orz cloudorzi@gmail.com
 # @LastEditTime: 2025-12-13 12:51:11
 # @FilePath: /udx710-backend/scripts/pack-ota.sh
 # @Description: 
 # 
 # Copyright (c) 2025 by 1orz, All Rights Reserved. 
### 
# 打包 OTA 更新包
# 输出: release/udx710-ota-{version}.tar.gz

set -e

# 切换到项目根目录
cd "$(dirname "$0")/.."

echo "=========================================="
echo "  打包 OTA 更新包"
echo "=========================================="
echo ""

# 读取版本号
VERSION_FILE="VERSION"
if [ -f "$VERSION_FILE" ]; then
    VERSION=$(cat "$VERSION_FILE" | tr -d '[:space:]')
else
    echo "❌ 错误: VERSION 文件不存在"
    exit 1
fi

# 获取 Git commit
if command -v git &> /dev/null && [ -d ".git" ]; then
    COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
else
    COMMIT="unknown"
fi

# 构建时间
BUILD_TIME=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# 目标架构
ARCH="aarch64-unknown-linux-gnu"

# 检查构建产物
BINARY_PATH="backend/target/aarch64-unknown-linux-gnu/release/udx710"
FRONTEND_DIR="frontend/dist"

if [ ! -f "$BINARY_PATH" ]; then
    echo "❌ 错误: 后端二进制不存在: $BINARY_PATH"
    echo "请先运行: ./scripts/build.sh"
    exit 1
fi

if [ ! -d "$FRONTEND_DIR" ]; then
    echo "❌ 错误: 前端构建产物不存在: $FRONTEND_DIR"
    echo "请先运行: ./scripts/build.sh"
    exit 1
fi

# 创建临时目录
OTA_TMP=$(mktemp -d)
trap "rm -rf $OTA_TMP" EXIT

echo "📦 版本: $VERSION"
echo "📝 Commit: $COMMIT"
echo "🕐 构建时间: $BUILD_TIME"
echo ""

# 复制后端二进制
echo "📋 复制后端二进制..."
cp "$BINARY_PATH" "$OTA_TMP/udx710"
chmod 755 "$OTA_TMP/udx710"

# 计算二进制 MD5
if [[ "$OSTYPE" == "darwin"* ]]; then
    BINARY_MD5=$(md5 -q "$OTA_TMP/udx710")
else
    BINARY_MD5=$(md5sum "$OTA_TMP/udx710" | cut -d' ' -f1)
fi
echo "   MD5: $BINARY_MD5"

# 复制前端文件
echo "📋 复制前端文件..."
mkdir -p "$OTA_TMP/www"
cp -r "$FRONTEND_DIR"/* "$OTA_TMP/www/"

# 计算前端 MD5（所有文件的 hash，与 Rust 验证逻辑一致）
# 方式：每个文件的 MD5 排序后，用换行符连接，再计算整体 MD5
echo "📋 计算前端 MD5..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS: 收集所有 MD5，排序，每行一个，然后计算整体 MD5
    FRONTEND_MD5=$(find "$OTA_TMP/www" -type f -exec md5 -q {} \; | sort | tr '\n' '\n' | md5 -q)
else
    # Linux: 同样的逻辑
    FRONTEND_MD5=$(find "$OTA_TMP/www" -type f -exec md5sum {} \; | cut -d' ' -f1 | sort | md5sum | cut -d' ' -f1)
fi
echo "   MD5: $FRONTEND_MD5"

# 生成 meta.json
echo "📋 生成 meta.json..."
cat > "$OTA_TMP/meta.json" << EOF
{
    "version": "$VERSION",
    "commit": "$COMMIT",
    "build_time": "$BUILD_TIME",
    "binary_md5": "$BINARY_MD5",
    "frontend_md5": "$FRONTEND_MD5",
    "arch": "$ARCH"
}
EOF

cat "$OTA_TMP/meta.json"
echo ""

# 创建输出目录
mkdir -p release

# 打包
OTA_FILE="release/udx710-ota-${VERSION}.tar.gz"
echo "📦 打包 OTA 更新包..."
cd "$OTA_TMP"
tar -czf - meta.json udx710 www > "$OLDPWD/$OTA_FILE"
cd "$OLDPWD"

# 显示结果
echo ""
echo "=========================================="
echo "✅ OTA 更新包打包完成！"
echo "=========================================="
echo ""
echo "📍 输出文件: $OTA_FILE"
ls -lh "$OTA_FILE"
echo ""
echo "📋 包内容:"
tar -tzf "$OTA_FILE" | head -20
echo "..."
echo ""

# 计算包的 MD5
if [[ "$OSTYPE" == "darwin"* ]]; then
    OTA_MD5=$(md5 -q "$OTA_FILE")
else
    OTA_MD5=$(md5sum "$OTA_FILE" | cut -d' ' -f1)
fi
echo "📝 OTA 包 MD5: $OTA_MD5"

