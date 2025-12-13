#!/bin/bash
###
 # @Author: 1orz cloudorzi@gmail.com
 # @Date: 2025-12-07 07:33:11
 # @LastEditors: 1orz cloudorzi@gmail.com
 # @LastEditTime: 2025-12-13 12:51:16
 # @FilePath: /udx710-backend/scripts/setup-env.sh
 # @Description: 
 # 
 # Copyright (c) 2025 by 1orz, All Rights Reserved. 
### 
# 
# R106 项目环境自动配置脚本
# 适用于 macOS (Apple Silicon 或 Intel)
#
# 使用方法: ./setup-env.sh
#

set -e

echo "=========================================="
echo "  R106 交叉编译环境配置脚本"
echo "=========================================="
echo ""

# 检测操作系统
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "❌ 错误：此脚本仅支持 macOS"
    exit 1
fi

# 检查 Homebrew
echo "📦 检查 Homebrew..."
if ! command -v brew &> /dev/null; then
    echo "❌ 未检测到 Homebrew，请先安装："
    echo "   /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
    exit 1
else
    echo "✅ Homebrew 已安装: $(brew --version | head -n 1)"
fi
echo ""

# 安装 rust
echo "🦀 安装 Rust..."
if brew list rust &>/dev/null; then
    echo "⚠️  Rust 已安装，跳过"
else
    brew install rust
    echo "✅ Rust 安装完成"
fi
echo ""

# 安装 rustup
echo "🔧 安装 rustup..."
if brew list rustup &>/dev/null; then
    echo "⚠️  rustup 已安装，跳过"
else
    brew install rustup
    echo "✅ rustup 安装完成"
fi
echo ""

# 配置 PATH
echo "⚙️  配置环境变量..."
RUSTUP_PATH='export PATH="/opt/homebrew/opt/rustup/bin:$PATH"'
if grep -q "opt/rustup/bin" ~/.zshrc; then
    echo "⚠️  PATH 已配置，跳过"
else
    echo "$RUSTUP_PATH" >> ~/.zshrc
    echo "✅ PATH 已添加到 ~/.zshrc"
fi

# 加载环境变量
export PATH="/opt/homebrew/opt/rustup/bin:$PATH"
echo ""

# 初始化 rustup
echo "🛠️  初始化 rustup..."
if rustup toolchain list | grep -q "stable"; then
    echo "⚠️  stable 工具链已存在，跳过"
else
    rustup default stable
    echo "✅ stable 工具链已设置为默认"
fi
echo ""

# 添加交叉编译目标
echo "🎯 添加 aarch64-unknown-linux-gnu 目标..."
if rustup target list --installed | grep -q "aarch64-unknown-linux-gnu"; then
    echo "⚠️  目标已安装，跳过"
else
    rustup target add aarch64-unknown-linux-gnu
    echo "✅ aarch64-unknown-linux-gnu 目标已添加"
fi
echo ""

# 添加交叉编译工具链 tap
echo "📚 添加交叉编译工具链仓库..."
if brew tap | grep -q "messense/macos-cross-toolchains"; then
    echo "⚠️  tap 已添加，跳过"
else
    brew tap messense/macos-cross-toolchains
    echo "✅ tap 已添加"
fi
echo ""

# 安装交叉编译工具链
echo "🔗 安装 aarch64-unknown-linux-gnu 工具链..."
if brew list aarch64-unknown-linux-gnu &>/dev/null; then
    echo "⚠️  工具链已安装，跳过"
else
    brew install aarch64-unknown-linux-gnu
    echo "✅ aarch64-unknown-linux-gnu 工具链已安装"
fi
echo ""

# 验证安装
echo "=========================================="
echo "  验证安装"
echo "=========================================="
echo ""

echo "📝 rustup 版本:"
rustup --version
echo ""

echo "📝 rustc 版本:"
rustc --version
echo ""

echo "📝 已安装的目标平台:"
rustup target list --installed
echo ""

echo "📝 交叉编译链接器:"
if command -v aarch64-unknown-linux-gnu-gcc &> /dev/null; then
    echo "✅ $(which aarch64-unknown-linux-gnu-gcc)"
    aarch64-unknown-linux-gnu-gcc --version | head -n 1
else
    echo "❌ 未找到 aarch64-unknown-linux-gnu-gcc"
fi
echo ""

# 检查 Xcode Command Line Tools
echo "📝 检查 Xcode Command Line Tools:"
if command -v cc &> /dev/null; then
    echo "✅ $(which cc)"
    cc --version | head -n 1
else
    echo "⚠️  未安装，运行: xcode-select --install"
fi
echo ""

echo "=========================================="
echo "  ✅ 环境配置完成！"
echo "=========================================="
echo ""
echo "📌 下一步："
echo "   1. 重新打开终端或运行: source ~/.zshrc"
echo "   2. 进入项目目录: cd $(pwd)"
echo "   3. 构建项目: ./build-aarch64.sh"
echo ""
echo "📖 详细文档请查看: README.md"
echo ""

