use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, MouseEvent, KeyboardEvent};
use std::cell::RefCell;
use std::rc::Rc;
use crate::ecs::world::World;
use crate::ecs::resources::{ResourceManager, InputState};
use crate::utils::Vec2;
use log::{info, debug};

/// 入力ハンドラー
/// ユーザーの入力イベント（マウス、キーボード、タッチ）を処理する
pub struct InputHandler {
    canvas: HtmlCanvasElement,
    world: Rc<RefCell<World>>,
    resources: Rc<RefCell<ResourceManager>>,
    
    // イベントリスナーのクロージャを保持
    // ドロップされないように保持する必要がある
    _mouse_down_listener: Option<Closure<dyn FnMut(MouseEvent)>>,
    _mouse_up_listener: Option<Closure<dyn FnMut(MouseEvent)>>,
    _mouse_move_listener: Option<Closure<dyn FnMut(MouseEvent)>>,
    _key_down_listener: Option<Closure<dyn FnMut(KeyboardEvent)>>,
    _key_up_listener: Option<Closure<dyn FnMut(KeyboardEvent)>>,
}

impl InputHandler {
    /// 新しい入力ハンドラーを作成
    pub fn new(
        canvas: HtmlCanvasElement,
        world: Rc<RefCell<World>>,
        resources: Rc<RefCell<ResourceManager>>,
    ) -> Result<Self, JsValue> {
        Ok(Self {
            canvas,
            world,
            resources,
            _mouse_down_listener: None,
            _mouse_up_listener: None,
            _mouse_move_listener: None,
            _key_down_listener: None,
            _key_up_listener: None,
        })
    }
    
    /// 入力イベントハンドラーを登録
    pub fn register_event_handlers(&self) -> Result<(), JsValue> {
        self.register_mouse_handlers()?;
        self.register_keyboard_handlers()?;
        
        info!("🖱️ 入力イベントハンドラーを登録しました");
        Ok(())
    }
    
    /// マウスイベントハンドラーを登録
    fn register_mouse_handlers(&self) -> Result<(), JsValue> {
        // mousedownイベントのハンドラーを作成
        let _world = Rc::clone(&self.world);
        let resources = Rc::clone(&self.resources);
        let canvas = self.canvas.clone();
        
        let mouse_down_closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            // イベントのデフォルト動作を防止
            event.prevent_default();
            
            // マウス座標を取得（キャンバス座標系に変換）
            let rect = canvas.get_bounding_client_rect();
            let x = event.client_x() as f64 - rect.left();
            let y = event.client_y() as f64 - rect.top();
            
            // 入力状態を更新
            if let Some(input_state) = resources.borrow_mut().get_mut::<InputState>() {
                input_state.update_mouse_position(x, y);
                input_state.update_mouse_button(0, true);  // 左ボタン
                input_state.is_mouse_clicked = true;  // クリックフラグを設定
                debug!("🖱️ マウスダウン: ({}, {})", x, y);
            }
        }) as Box<dyn FnMut(MouseEvent)>);
        
        // mouseupイベントのハンドラーを作成
        let _world_up = Rc::clone(&self.world);
        let resources_up = Rc::clone(&self.resources);
        let canvas_up = self.canvas.clone();
        
        let mouse_up_closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            event.prevent_default();
            
            let rect = canvas_up.get_bounding_client_rect();
            let x = event.client_x() as f64 - rect.left();
            let y = event.client_y() as f64 - rect.top();
            
            if let Some(input_state) = resources_up.borrow_mut().get_mut::<InputState>() {
                input_state.update_mouse_position(x, y);
                input_state.update_mouse_button(0, false);  // 左ボタン
                debug!("🖱️ マウスアップ: ({}, {})", x, y);
            }
        }) as Box<dyn FnMut(MouseEvent)>);
        
        // mousemoveイベントのハンドラーを作成
        let _world_move = Rc::clone(&self.world);
        let resources_move = Rc::clone(&self.resources);
        let canvas_move = self.canvas.clone();
        
        let mouse_move_closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            // マウス移動イベントは頻繁に発生するのでpreventDefaultは不要
            
            let rect = canvas_move.get_bounding_client_rect();
            let x = event.client_x() as f64 - rect.left();
            let y = event.client_y() as f64 - rect.top();
            
            if let Some(input_state) = resources_move.borrow_mut().get_mut::<InputState>() {
                input_state.update_mouse_position(x, y);
            }
        }) as Box<dyn FnMut(MouseEvent)>);
        
        // キャンバスにイベントリスナーを追加
        self.canvas.add_event_listener_with_callback(
            "mousedown",
            mouse_down_closure.as_ref().unchecked_ref(),
        )?;
        
        self.canvas.add_event_listener_with_callback(
            "mouseup",
            mouse_up_closure.as_ref().unchecked_ref(),
        )?;
        
        self.canvas.add_event_listener_with_callback(
            "mousemove",
            mouse_move_closure.as_ref().unchecked_ref(),
        )?;
        
        // クロージャを保持（ドロップされないように）
        let this = self as *const _ as *mut InputHandler;
        unsafe {
            (*this)._mouse_down_listener = Some(mouse_down_closure);
            (*this)._mouse_up_listener = Some(mouse_up_closure);
            (*this)._mouse_move_listener = Some(mouse_move_closure);
        }
        
        Ok(())
    }
    
    /// キーボードイベントハンドラーを登録
    fn register_keyboard_handlers(&self) -> Result<(), JsValue> {
        // キーボードイベントはドキュメント全体に設定
        let document = web_sys::window()
            .ok_or_else(|| JsValue::from_str("ウィンドウが見つかりません"))?
            .document()
            .ok_or_else(|| JsValue::from_str("ドキュメントが見つかりません"))?;
        
        // keydownイベントのハンドラーを作成
        let resources_down = Rc::clone(&self.resources);
        
        let key_down_closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            let key = event.key();
            
            if let Some(input_state) = resources_down.borrow_mut().get_mut::<InputState>() {
                input_state.update_key(&key, true);
                debug!("⌨️ キーダウン: {}", key);
            }
        }) as Box<dyn FnMut(KeyboardEvent)>);
        
        // keyupイベントのハンドラーを作成
        let resources_up = Rc::clone(&self.resources);
        
        let key_up_closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            let key = event.key();
            
            if let Some(input_state) = resources_up.borrow_mut().get_mut::<InputState>() {
                input_state.update_key(&key, false);
                debug!("⌨️ キーアップ: {}", key);
            }
        }) as Box<dyn FnMut(KeyboardEvent)>);
        
        // ドキュメントにイベントリスナーを追加
        document.add_event_listener_with_callback(
            "keydown",
            key_down_closure.as_ref().unchecked_ref(),
        )?;
        
        document.add_event_listener_with_callback(
            "keyup",
            key_up_closure.as_ref().unchecked_ref(),
        )?;
        
        // クロージャを保持（ドロップされないように）
        let this = self as *const _ as *mut InputHandler;
        unsafe {
            (*this)._key_down_listener = Some(key_down_closure);
            (*this)._key_up_listener = Some(key_up_closure);
        }
        
        Ok(())
    }
    
    /// 指定した座標にあるエンティティを取得
    pub fn get_entity_at_position(
        world: &World,
        position: Vec2,
    ) -> Option<usize> {
        // レンダラブルコンポーネントを持つエンティティのうち、
        // Z-indexが大きい（上に表示されている）順にソート
        let mut entities = world.get_entities_with_component::<crate::ecs::component::Renderable>();
        
        if entities.is_empty() {
            return None;
        }
        
        // Z-indexでソート（大きい順）
        entities.sort_by(|&a, &b| {
            let z_a = world
                .get_component::<crate::ecs::component::Transform>(a)
                .map(|t| t.z_index)
                .unwrap_or(0);
                
            let z_b = world
                .get_component::<crate::ecs::component::Transform>(b)
                .map(|t| t.z_index)
                .unwrap_or(0);
                
            z_b.cmp(&z_a)  // 降順
        });
        
        // 座標が含まれるエンティティを探す
        for &entity_id in &entities {
            let transform = match world.get_component::<crate::ecs::component::Transform>(entity_id) {
                Some(t) => t,
                None => continue,
            };
            
            let renderable = match world.get_component::<crate::ecs::component::Renderable>(entity_id) {
                Some(r) => r,
                None => continue,
            };
            
            // エンティティの領域内にあるかチェック
            let x = transform.position.x;
            let y = transform.position.y;
            let width = renderable.width;
            let height = renderable.height;
            
            if position.x >= x && position.x <= x + width &&
               position.y >= y && position.y <= y + height {
                return Some(entity_id);
            }
        }
        
        None
    }
} 