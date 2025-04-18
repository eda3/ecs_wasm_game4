use wasm_bindgen::prelude::*;
use log::info;

// モジュール定義
mod ecs;
mod game;
mod render;
mod input;
mod network;
mod utils;
mod constants;
mod components;
mod resources;

use crate::game::Game;

/// ゲーム初期化関数
/// Javascriptからこの関数を呼び出してゲームを開始する
#[wasm_bindgen(start)]
pub fn start() {
    // パニック時にエラーメッセージをコンソールに表示するフックを設定
    utils::set_panic_hook();
    // Rustのロガーを初期化
    wasm_logger::init(wasm_logger::Config::default());
    
    info!("🎮 ソリティアゲーム WebAssembly版を初期化中... 🎮");
}

/// ゲームを初期化するJavaScript向け関数
#[wasm_bindgen]
pub fn init_game() {
    info!("init_game()が呼び出されました");
    // 将来的に初期化ロジックをここに追加
}

/// 新しいゲームを作成するJavaScript向け関数
#[wasm_bindgen]
pub fn create_game(canvas_id: &str) -> Result<Game, JsValue> {
    info!("create_game({})が呼び出されました", canvas_id);
    let mut game = Game::new(canvas_id)?;
    
    // ゲームを開始
    game.start()?;
    
    Ok(game)
}

/// 新しいゲームを開始するJavaScript向け関数
#[wasm_bindgen]
pub fn new_game() {
    info!("new_game()が呼び出されました");
    // 将来的に新規ゲーム開始ロジックをここに追加
}

/// 操作を元に戻すJavaScript向け関数
#[wasm_bindgen]
pub fn undo_move() {
    info!("undo_move()が呼び出されました");
    // 将来的にundo機能をここに追加
}

/// ゲーム状態を更新するJavaScript向け関数
#[wasm_bindgen]
pub fn update_game_state(state_json: &str) {
    info!("update_game_state()が呼び出されました: {}", state_json);
    // 将来的に状態更新ロジックをここに追加
}

/// クリック位置を処理するJavaScript向け関数
#[wasm_bindgen]
pub fn handle_click(x: f64, y: f64) {
    info!("handle_click({}, {})が呼び出されました", x, y);
    // 将来的にクリック処理ロジックをここに追加
}

/// テスト用Hello関数
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("こんにちは、{}さん！WebAssemblyのソリティアゲームへようこそ！🎮✨", name)
} 