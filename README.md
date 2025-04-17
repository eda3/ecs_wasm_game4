# ECS Wasm ゲーム

Rust と WebAssembly を使用した ECS アーキテクチャベースのソリティアゲーム

## 必要なツール

- [Rust](https://www.rust-lang.org/tools/install) - 1.70以上推奨
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) - WebAssemblyコンパイラ
- [Node.js](https://nodejs.org/) - 16.x以上推奨

## ビルド方法

プロジェクトのルートディレクトリで以下のコマンドを実行してください：

```bash
./build.sh
```

このスクリプトは以下の処理を行います：
1. Rustコードを WebAssembly にコンパイル
2. フロントエンドの依存関係をインストール

## 実行方法

ビルド後、以下のコマンドでサーバーを起動できます：

```bash
npm run dev
```

ブラウザで http://162.43.8.148:8001 にアクセスしてゲームを開始できます。

## プロジェクト構造

- `src/` - Rustのソースコード
  - `ecs/` - Entity Component System の実装
  - `game/` - ゲームロジック
  - `render/` - 描画関連のコード
- `www/` - WebフロントエンドとHTTP/WebSocketサーバー
  - `server.js` - Express サーバーとWebSocketの実装
  - `index.html` - ゲームのHTMLとフロントエンドのJavaScript 