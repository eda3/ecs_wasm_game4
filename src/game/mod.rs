// ゲームモジュール
//
// このモジュールは、ソリティアゲーム全体の管理とゲームプレイの流れを制御します。
// ECSアーキテクチャを使って、カード、スタック、ルールなどを実装します。

// サブモジュール
pub mod card;        // カード関連
pub mod solitaire;   // ソリティアゲームのルール
pub mod setup;       // ゲーム初期化
pub mod state;       // ゲーム状態管理

// 他のモジュールからのインポート
use wasm_bindgen::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d};
use crate::ecs::world::World;
use crate::ecs::system::SystemManager;
use crate::ecs::resources::ResourceManager;
use crate::render::renderer::Renderer;
use crate::input::input_handler::InputHandler;
use crate::game::setup::setup_game;
use log::{info, error};

// ゲームのメインループを処理するクロージャの型
type GameLoopCallback = Closure<dyn FnMut(f64)>;

/// ゲームを管理する構造体
/// WebAssemblyからJavaScriptへエクスポートされる
#[wasm_bindgen]
pub struct Game {
    // キャンバス要素
    canvas: HtmlCanvasElement,
    
    // 描画コンテキスト
    context: CanvasRenderingContext2d,
    
    // ECSのワールド
    world: Rc<RefCell<World>>,
    
    // システムマネージャー
    system_manager: Rc<RefCell<SystemManager>>,
    
    // リソースマネージャー
    resource_manager: Rc<RefCell<ResourceManager>>,
    
    // レンダラー
    renderer: Renderer,
    
    // 入力ハンドラー
    input_handler: InputHandler,
    
    // ゲームループコールバック
    // Closureをドロップするとコールバックが停止するため、保持しておく
    _game_loop: Option<GameLoopCallback>,
    
    // ゲームが実行中かどうか
    is_running: bool,
}

#[wasm_bindgen]
impl Game {
    /// 新しいゲームを作成
    pub fn new(canvas_id: &str) -> Result<Game, JsValue> {
        info!("🎮 新しいゲームを作成中... canvas_id: {}", canvas_id);
        
        // DOMからキャンバス要素を取得
        let window = web_sys::window()
            .ok_or_else(|| {
                let err_msg = "ウィンドウが見つかりません";
                error!("エラー: {}", err_msg);
                JsValue::from_str(err_msg)
            })?;
            
        let document = window
            .document()
            .ok_or_else(|| {
                let err_msg = "ドキュメントが見つかりません";
                error!("エラー: {}", err_msg);
                JsValue::from_str(err_msg)
            })?;
        
        info!("キャンバス要素を検索中: #{}", canvas_id);
        let canvas_element = document.get_element_by_id(canvas_id);
        
        if canvas_element.is_none() {
            let err_msg = format!("ID: '{}' のキャンバス要素が見つかりません。HTMLに対応する要素が存在することを確認してください。", canvas_id);
            error!("エラー: {}", err_msg);
            return Err(JsValue::from_str(&err_msg));
        }
        
        let canvas = canvas_element
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| {
                let err_msg = format!("ID: '{}' の要素はHtmlCanvasElementではありません", canvas_id);
                error!("エラー: {}", err_msg);
                JsValue::from_str(&err_msg)
            })?;
            
        info!("キャンバス要素を取得しました: {}x{}", canvas.width(), canvas.height());
        
        // キャンバスサイズを設定
        canvas.set_width(crate::constants::CANVAS_WIDTH);
        canvas.set_height(crate::constants::CANVAS_HEIGHT);
        
        // 2Dコンテキストを取得
        let context = canvas
            .get_context("2d")?
            .ok_or_else(|| {
                let err_msg = "2Dコンテキストを取得できません";
                error!("エラー: {}", err_msg);
                JsValue::from_str(err_msg)
            })?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| {
                let err_msg = "コンテキストをCanvasRenderingContext2dに変換できません";
                error!("エラー: {}", err_msg);
                JsValue::from_str(err_msg)
            })?;
            
        // ECSコンポーネントを初期化
        let world = Rc::new(RefCell::new(World::new()));
        let system_manager = Rc::new(RefCell::new(SystemManager::new()));
        let resource_manager = Rc::new(RefCell::new(ResourceManager::new()));
        
        // レンダラーと入力ハンドラーを初期化
        let renderer = Renderer::new(canvas.clone(), context.clone());
        let input_handler = InputHandler::new(canvas.clone(), Rc::clone(&world), Rc::clone(&resource_manager))?;
        
        // ゲームを初期化
        setup_game(
            &mut world.borrow_mut(),
            &mut system_manager.borrow_mut(),
            &mut resource_manager.borrow_mut(),
        )?;
        
        info!("✨ ゲームの初期化が完了しました！");
        
        Ok(Game {
            canvas,
            context,
            world,
            system_manager,
            resource_manager,
            renderer,
            input_handler,
            _game_loop: None,
            is_running: false,
        })
    }
    
    /// ゲームを開始
    pub fn start(&mut self) -> Result<(), JsValue> {
        if self.is_running {
            return Ok(());  // 既に実行中の場合は何もしない
        }
        
        info!("🚀 ゲームを開始します！");
        self.is_running = true;
        
        // JavaScriptのウィンドウオブジェクトを取得
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("ウィンドウが見つかりません"))?;
        
        // ゲームループのクロージャを作成
        let world_clone = Rc::clone(&self.world);
        let system_manager_clone = Rc::clone(&self.system_manager);
        let resource_manager_clone = Rc::clone(&self.resource_manager);
        let renderer_clone = self.renderer.clone();
        
        // レンダリングコールバックを作成
        let f = Rc::new(RefCell::new(None::<Closure<dyn FnMut(f64)>>));
        let g = Rc::clone(&f);
        
        // ゲームループのクロージャを定義
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move |timestamp: f64| {
            // ゲームの更新とレンダリングを行う
            let mut world = world_clone.borrow_mut();
            let mut system_manager = system_manager_clone.borrow_mut();
            let mut resource_manager = resource_manager_clone.borrow_mut();
            
            // 時間情報を更新
            if let Some(time_info) = resource_manager.get_mut::<crate::ecs::resources::TimeInfo>() {
                time_info.update(timestamp);
                let delta_time = time_info.delta_time;
                
                // システムを実行（ゲームの更新）
                if let Err(e) = world.run_systems(&mut system_manager, &mut resource_manager, delta_time) {
                    error!("システムの実行中にエラーが発生しました: {:?}", e);
                }
            }
            
            // レンダリング
            if let Err(e) = renderer_clone.render(&world, &resource_manager) {
                error!("レンダリング中にエラーが発生しました: {:?}", e);
            }
            
            // 次のフレームをリクエスト
            if let Some(ref callback) = *f.borrow() {
                window.request_animation_frame(callback.as_ref().unchecked_ref()).unwrap();
            }
        }) as Box<dyn FnMut(f64)>));
        
        // 最初のアニメーションフレームをリクエストするために新しいウィンドウインスタンスを取得
        let window_for_first_request = web_sys::window().ok_or_else(|| JsValue::from_str("最初のリクエスト用ウィンドウが見つかりません"))?;
        
        // アニメーションフレームをリクエスト
        window_for_first_request.request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())?;
        
        // クロージャをドロップされないように保持
        self._game_loop = Some(g.borrow_mut().take().unwrap());
        
        Ok(())
    }
    
    /// ゲームを停止
    pub fn stop(&mut self) {
        info!("⏹️ ゲームを停止します");
        self.is_running = false;
        self._game_loop = None;  // クロージャをドロップしてゲームループを停止
    }
    
    /// ゲームをリセット
    pub fn reset(&mut self) -> Result<(), JsValue> {
        info!("🔄 ゲームをリセットします");
        
        // ゲームを一時停止
        let was_running = self.is_running;
        self.stop();
        
        // ワールドとリソースをクリア
        self.world.borrow_mut().clear();
        
        // ゲームを再初期化
        setup_game(
            &mut self.world.borrow_mut(),
            &mut self.system_manager.borrow_mut(),
            &mut self.resource_manager.borrow_mut(),
        )?;
        
        // 実行中だった場合は再開
        if was_running {
            self.start()?;
        }
        
        Ok(())
    }
    
    /// 入力イベントを登録
    pub fn setup_input_handlers(&self) -> Result<(), JsValue> {
        self.input_handler.register_event_handlers()
    }
}

// Dropトレイトを実装して、リソースの解放を行う
impl Drop for Game {
    fn drop(&mut self) {
        info!("👋 ゲームを終了します");
        self.stop();  // ゲームループを停止
        
        // ここで追加のクリーンアップが必要な場合は実装
    }
} 