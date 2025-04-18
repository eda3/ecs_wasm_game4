use wasm_bindgen::prelude::*;
use log::{info, error};
use std::cell::RefCell;
use std::rc::Rc;

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
use crate::utils::Vec2;

// グローバルなゲームインスタンス
thread_local! {
    static GAME_INSTANCE: RefCell<Option<Game>> = RefCell::new(None);
}

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
    
    // グローバルインスタンスを保存
    GAME_INSTANCE.with(|instance| {
        *instance.borrow_mut() = Some(game.clone());
    });
    
    Ok(game)
}

/// 新しいゲームを開始するJavaScript向け関数
#[wasm_bindgen]
pub fn new_game() {
    info!("new_game()が呼び出されました");
    GAME_INSTANCE.with(|instance| {
        if let Some(ref mut game) = *instance.borrow_mut() {
            if let Err(e) = game.reset() {
                error!("ゲームのリセット中にエラーが発生しました: {:?}", e);
            }
        }
    });
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
    
    // グローバルなゲームインスタンスがあれば、クリックイベントを処理
    GAME_INSTANCE.with(|instance| {
        if let Some(ref game) = *instance.borrow() {
            // ゲームにクリックイベントを処理させる
            if let Some(entity_id) = game.handle_entity_click(x, y) {
                info!("エンティティID {} がクリックされました", entity_id);
            }
        }
    });
}

/// テスト用Hello関数
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("こんにちは、{}さん！WebAssemblyのソリティアゲームへようこそ！🎮✨", name)
} 