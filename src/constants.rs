// ゲームの定数

// ゲームのフレームレート関連
pub const TARGET_FPS: u32 = 60;
pub const FRAME_TIME_MS: f64 = 1000.0 / TARGET_FPS as f64;

// キャンバスサイズ
pub const CANVAS_WIDTH: u32 = 800;
pub const CANVAS_HEIGHT: u32 = 600;

// カードサイズ
pub const CARD_WIDTH: f64 = 80.0;
pub const CARD_HEIGHT: f64 = 120.0;
pub const CARD_BORDER_RADIUS: f64 = 5.0;

// カードの間隔
pub const CARD_SPACING_X: f64 = 20.0;
pub const CARD_SPACING_Y: f64 = 30.0;
pub const STACK_OFFSET_Y: f64 = 25.0;

// カードの色
pub const CARD_BACK_COLOR: &str = "#2C3E50";
pub const CARD_FRONT_COLOR: &str = "#FFFFFF";
pub const CARD_BORDER_COLOR: &str = "#34495E";
pub const CARD_TEXT_COLOR: &str = "#2C3E50";
pub const CARD_RED_COLOR: &str = "#E74C3C";
pub const CARD_BLACK_COLOR: &str = "#2C3E50";

// ゲームエリアのレイアウト
pub const FOUNDATION_START_X: f64 = 400.0;
pub const FOUNDATION_START_Y: f64 = 50.0;
pub const TABLEAU_START_X: f64 = 100.0;
pub const TABLEAU_START_Y: f64 = 200.0;
pub const STOCK_X: f64 = 100.0;
pub const STOCK_Y: f64 = 50.0;
pub const WASTE_X: f64 = 200.0;
pub const WASTE_Y: f64 = 50.0;

// ドラッグ関連
pub const DRAG_OPACITY: f64 = 0.7;
pub const ANIMATION_DURATION: f64 = 300.0; // ミリ秒

// ネットワーク
pub const WS_SERVER_URL: &str = "ws://localhost:8080";

// ドラッグアンドドロップの閾値（ピクセル）
pub const DRAG_THRESHOLD: f64 = 5.0;

// ECS関連の定数
pub const MAX_ENTITIES: usize = 1000;

// カードのスート（マーク）
pub const SUIT_HEART: u8 = 0;
pub const SUIT_DIAMOND: u8 = 1;
pub const SUIT_CLUB: u8 = 2;
pub const SUIT_SPADE: u8 = 3;

// カードスートの文字表現
pub const SUIT_SYMBOLS: [&str; 4] = ["♥", "♦", "♣", "♠"];

// カードの数字の文字表現
pub const RANK_SYMBOLS: [&str; 13] = ["A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K"]; 