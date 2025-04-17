// ゲームの定数
// このファイルには、ゲーム全体で使用する様々な定数値が定義されています。
// 数値を一箇所にまとめることで、ゲームの調整や変更が容易になります。

//
// ゲームの基本設定
//

// ゲームのフレームレート関連
// TARGET_FPSはゲームの目標フレームレート（1秒間に描画する回数）
pub const TARGET_FPS: u32 = 60;
// FRAME_TIME_MSは1フレームあたりの理想的な時間（ミリ秒）
pub const FRAME_TIME_MS: f64 = 1000.0 / TARGET_FPS as f64;

//
// 画面・表示関連
//

// キャンバスサイズ（ピクセル単位）
pub const CANVAS_WIDTH: u32 = 800;
pub const CANVAS_HEIGHT: u32 = 600;

// カードサイズ（ピクセル単位）
pub const CARD_WIDTH: f64 = 80.0;
pub const CARD_HEIGHT: f64 = 120.0;
pub const CARD_BORDER_RADIUS: f64 = 5.0;  // カード角の丸み

// カードの間隔と配置
pub const CARD_SPACING_X: f64 = 20.0;  // カード間の横方向の間隔
pub const CARD_SPACING_Y: f64 = 30.0;  // カード間の縦方向の間隔
pub const STACK_OFFSET_Y: f64 = 25.0;  // 重なったカードの表示オフセット

// カードの色設定（HTMLカラーコード）
pub const CARD_BACK_COLOR: &str = "#2C3E50";  // カード裏面の色
pub const CARD_FRONT_COLOR: &str = "#FFFFFF";  // カード表面の色
pub const CARD_BORDER_COLOR: &str = "#34495E";  // カードの枠線色
pub const CARD_TEXT_COLOR: &str = "#2C3E50";  // カードの文字色
pub const CARD_RED_COLOR: &str = "#E74C3C";  // 赤いカード（ハートとダイヤ）の色
pub const CARD_BLACK_COLOR: &str = "#2C3E50";  // 黒いカード（クラブとスペード）の色

//
// ゲームレイアウト関連（座標）
//

// ソリティアのレイアウト設定 - 画面上の位置（ピクセル単位）
pub const FOUNDATION_START_X: f64 = 400.0;  // 組み札（右上）の開始X座標
pub const FOUNDATION_START_Y: f64 = 50.0;   // 組み札（右上）の開始Y座標
pub const TABLEAU_START_X: f64 = 100.0;     // 場札（中央）の開始X座標
pub const TABLEAU_START_Y: f64 = 200.0;     // 場札（中央）の開始Y座標
pub const STOCK_X: f64 = 100.0;             // 山札（左上）のX座標
pub const STOCK_Y: f64 = 50.0;              // 山札（左上）のY座標
pub const WASTE_X: f64 = 200.0;             // 捨て札（山札の右）のX座標
pub const WASTE_Y: f64 = 50.0;              // 捨て札（山札の右）のY座標

//
// アニメーションと視覚効果
//

// ドラッグ関連の設定
pub const DRAG_OPACITY: f64 = 0.7;  // ドラッグ中のカードの透明度
pub const ANIMATION_DURATION: f64 = 300.0; // アニメーション時間（ミリ秒）

//
// ネットワーク設定
//

// WebSocketサーバーのURL
pub const WS_SERVER_URL: &str = "ws://162.43.8.148:8101";  // WebSocketサーバーのURL（IPアドレス:ポート8101）

// ドラッグアンドドロップの閾値（ピクセル単位）
// この値より大きく動かすと、クリックではなくドラッグとして認識
pub const DRAG_THRESHOLD: f64 = 5.0;

//
// ECS関連の定数
//

// エンティティの最大数
pub const MAX_ENTITIES: usize = 1000;

//
// カード関連の定数
//

// カードのスート（マーク）の数値表現
pub const SUIT_HEART: u8 = 0;    // ハート
pub const SUIT_DIAMOND: u8 = 1;  // ダイヤ
pub const SUIT_CLUB: u8 = 2;     // クラブ
pub const SUIT_SPADE: u8 = 3;    // スペード

// カードスートの文字表現
pub const SUIT_SYMBOLS: [&str; 4] = ["♥", "♦", "♣", "♠"];

// カードの数字の文字表現
pub const RANK_SYMBOLS: [&str; 13] = ["A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K"]; 