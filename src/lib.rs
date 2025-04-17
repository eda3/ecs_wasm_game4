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

use crate::ecs::world::World;
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

/// 新しいゲームを作成するJavaScript向け関数
#[wasm_bindgen]
pub fn create_game(canvas_id: &str) -> Result<Game, JsValue> {
    let game = Game::new(canvas_id)?;
    Ok(game)
}

/// テスト用Hello関数
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("こんにちは、{}さん！WebAssemblyのソリティアゲームへようこそ！🎮✨", name)
} 