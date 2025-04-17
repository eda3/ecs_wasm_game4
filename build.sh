#!/bin/bash

# エラー発生時に中止する
set -e

echo "🦀 ECS Wasmゲームのビルドを開始します 🎮"

# Rustプロジェクトをビルドしてwasmを生成
echo "📦 Rustコードをコンパイルしています..."
wasm-pack build --target web --out-dir ./www/pkg

# フロントエンドの依存関係をインストール
echo "📚 フロントエンドの依存関係をインストールしています..."
cd www
npm install

# フロントエンドのビルド（必要に応じて）
# npm run build

echo "✅ ビルドが完了しました！"
echo "🚀 サーバーを起動するには 'npm run dev' を実行してください" 