#!/bin/bash

# エラー発生時に中止する
set -e

echo "🚀 ECS Wasmゲームのデプロイを開始します"

# 既存のサーバープロセスを停止
pkill -f "node server.js" || true
echo "📋 既存のサーバープロセスを停止しました"

# ビルドスクリプトを実行
echo "🔨 ビルドを実行します"
./build.sh

echo "✅ デプロイが完了しました！"