use std::collections::HashMap;
use std::any::{Any, TypeId};
use wasm_bindgen::prelude::*;
use crate::utils::Vec2;

/// リソースマネージャー
/// グローバルな状態やシステム間で共有される情報を管理する
pub struct ResourceManager {
    // TypeIdからAny型へのマップ
    resources: HashMap<TypeId, Box<dyn Any>>,
}

impl ResourceManager {
    /// 新しいリソースマネージャーを作成
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }
    
    /// リソースを追加
    pub fn add<T: 'static>(&mut self, resource: T) {
        let type_id = TypeId::of::<T>();
        self.resources.insert(type_id, Box::new(resource));
    }
    
    /// リソースを取得
    pub fn get<T: 'static>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.resources
            .get(&type_id)
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }
    
    /// リソースを可変で取得
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.resources
            .get_mut(&type_id)
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }
    
    /// リソースを削除
    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        self.resources
            .remove(&type_id)
            .and_then(|boxed| boxed.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }
    
    /// リソースが存在するかチェック
    pub fn has<T: 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.resources.contains_key(&type_id)
    }
    
    /// リソースを取得、存在しない場合はデフォルト値を使って作成
    pub fn get_or_insert_with<T: 'static, F>(&mut self, f: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        let type_id = TypeId::of::<T>();
        
        if !self.resources.contains_key(&type_id) {
            let resource = f();
            self.resources.insert(type_id, Box::new(resource));
        }
        
        self.resources
            .get_mut(&type_id)
            .and_then(|boxed| boxed.downcast_mut::<T>())
            .unwrap()
    }
}

/// 入力状態を管理するリソース
#[derive(Default)]
pub struct InputState {
    pub mouse_position: Vec2,
    pub mouse_buttons: [bool; 3], // [左, 中, 右]
    pub mouse_down_position: Vec2,
    pub is_mouse_down: bool,
    pub is_mouse_clicked: bool,  // マウスクリックが発生したかどうか（1フレームだけtrue）
    pub keys_pressed: HashMap<String, bool>,
    pub touch_position: Vec2,
    pub is_touch_active: bool,
}

impl InputState {
    /// 新しい入力状態を作成
    pub fn new() -> Self {
        Self {
            mouse_position: Vec2::zero(),
            mouse_buttons: [false; 3],
            mouse_down_position: Vec2::zero(),
            is_mouse_down: false,
            is_mouse_clicked: false,
            keys_pressed: HashMap::new(),
            touch_position: Vec2::zero(),
            is_touch_active: false,
        }
    }
    
    /// マウスの位置を更新
    pub fn update_mouse_position(&mut self, x: f64, y: f64) {
        self.mouse_position = Vec2::new(x, y);
    }
    
    /// マウスボタンの状態を更新
    pub fn update_mouse_button(&mut self, button: usize, pressed: bool) {
        if button < self.mouse_buttons.len() {
            self.mouse_buttons[button] = pressed;
            
            if button == 0 {  // 左ボタン
                self.is_mouse_down = pressed;
                log::debug!("🖱️ マウスボタン状態更新: button={}, pressed={}, is_mouse_down={}", button, pressed, self.is_mouse_down);
                if pressed {
                    self.mouse_down_position = self.mouse_position;
                }
            }
        }
    }
    
    /// キーの状態を更新
    pub fn update_key(&mut self, key: &str, pressed: bool) {
        self.keys_pressed.insert(key.to_string(), pressed);
    }
    
    /// タッチの位置を更新
    pub fn update_touch(&mut self, x: f64, y: f64, is_active: bool) {
        self.touch_position = Vec2::new(x, y);
        self.is_touch_active = is_active;
        
        // タッチはマウスにも反映させる（シンプルな入力処理のため）
        self.mouse_position = self.touch_position;
        self.is_mouse_down = is_active;
        if is_active {
            self.mouse_down_position = self.touch_position;
        }
    }
    
    /// キーが押されているかチェック
    pub fn is_key_pressed(&self, key: &str) -> bool {
        *self.keys_pressed.get(key).unwrap_or(&false)
    }
    
    /// マウスが指定した矩形内にあるかチェック
    pub fn is_mouse_in_rect(&self, x: f64, y: f64, width: f64, height: f64) -> bool {
        self.mouse_position.x >= x
            && self.mouse_position.x <= x + width
            && self.mouse_position.y >= y
            && self.mouse_position.y <= y + height
    }
}

/// 時間関連情報を管理するリソース
pub struct TimeInfo {
    pub total_time: f64,     // ゲーム開始からの経過時間（秒）
    pub delta_time: f32,     // 前フレームからの経過時間（秒）
    pub frame_count: u64,    // フレーム数
    pub target_fps: u32,     // 目標フレームレート
    pub last_frame_time: f64, // 前フレームの時間（パフォーマンス計測用）
}

impl TimeInfo {
    /// 新しい時間情報を作成
    pub fn new(target_fps: u32) -> Self {
        Self {
            total_time: 0.0,
            delta_time: 0.0,
            frame_count: 0,
            target_fps,
            last_frame_time: 0.0,
        }
    }
    
    /// 時間情報を更新
    pub fn update(&mut self, current_time: f64) {
        // 前フレームからの経過時間を計算
        if self.last_frame_time > 0.0 {
            self.delta_time = ((current_time - self.last_frame_time) / 1000.0) as f32;
        } else {
            self.delta_time = 1.0 / self.target_fps as f32;
        }
        
        // 極端に大きなデルタタイムをクランプ（フレームレート低下時の対策）
        const MAX_DELTA_TIME: f32 = 0.1; // 100ミリ秒
        if self.delta_time > MAX_DELTA_TIME {
            self.delta_time = MAX_DELTA_TIME;
        }
        
        // 時間と統計を更新
        self.total_time += self.delta_time as f64;
        self.last_frame_time = current_time;
        self.frame_count += 1;
    }
    
    /// 現在のFPSを計算
    pub fn get_fps(&self) -> f64 {
        if self.delta_time > 0.0 {
            1.0 / self.delta_time as f64
        } else {
            0.0
        }
    }
}

/// ゲームの状態を管理するリソース
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GameState {
    /// タイトル画面
    Title,
    /// ゲームプレイ中
    Playing,
    /// 一時停止中
    Paused,
    /// ゲームオーバー
    GameOver,
    /// クリア（ゲーム完了）
    Clear,
}

/// ネットワーク状態を管理するリソース
pub struct NetworkState {
    pub is_connected: bool,
    pub player_id: Option<String>,
    pub other_players: Vec<String>,
    pub connection_error: Option<String>,
    pub last_message_time: f64,
}

impl NetworkState {
    /// 新しいネットワーク状態を作成
    pub fn new() -> Self {
        Self {
            is_connected: false,
            player_id: None,
            other_players: Vec::new(),
            connection_error: None,
            last_message_time: 0.0,
        }
    }
    
    /// 接続状態を更新
    pub fn set_connected(&mut self, connected: bool) {
        self.is_connected = connected;
        if !connected {
            self.player_id = None;
            self.other_players.clear();
        }
    }
    
    /// プレイヤーIDを設定
    pub fn set_player_id(&mut self, id: &str) {
        self.player_id = Some(id.to_string());
    }
    
    /// 他のプレイヤーを追加
    pub fn add_player(&mut self, id: &str) {
        if !self.other_players.contains(&id.to_string()) {
            self.other_players.push(id.to_string());
        }
    }
    
    /// プレイヤーを削除
    pub fn remove_player(&mut self, id: &str) {
        self.other_players.retain(|player_id| player_id != id);
    }
    
    /// エラーを設定
    pub fn set_error(&mut self, error: &str) {
        self.connection_error = Some(error.to_string());
    }
    
    /// エラーをクリア
    pub fn clear_error(&mut self) {
        self.connection_error = None;
    }
} 