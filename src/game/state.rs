use wasm_bindgen::prelude::*;
use crate::ecs::world::World;
use crate::ecs::system::{System, SystemPhase, SystemPriority};
use crate::ecs::resources::{ResourceManager, GameState};
use crate::ecs::component::{StackContainer, StackType};
use crate::game::solitaire;
use log::{info, debug};

/// ゲームの状態を管理するシステム
pub struct GameStateSystem {
    foundation_ids: Vec<usize>, // ファウンデーションのエンティティID
}

impl GameStateSystem {
    /// 新しいゲーム状態システムを作成
    pub fn new() -> Self {
        Self {
            foundation_ids: Vec::new(),
        }
    }
    
    /// ファウンデーションのエンティティIDを見つける
    fn find_foundation_ids(&mut self, world: &World) {
        if !self.foundation_ids.is_empty() {
            return;  // 既に見つかっている場合は何もしない
        }
        
        // StackContainerコンポーネントを持つエンティティを取得
        let entities_with_stack = world.get_entities_with_component::<StackContainer>();
        
        // ファウンデーションのエンティティを探す
        for entity_id in entities_with_stack {
            if let Some(stack) = world.get_component::<StackContainer>(entity_id) {
                match stack.stack_type {
                    StackType::Foundation { .. } => {
                        self.foundation_ids.push(entity_id);
                    },
                    _ => {},
                }
            }
        }
    }
    
    /// ゲームがクリアされたかチェック
    fn check_game_clear(&self, world: &World) -> bool {
        if self.foundation_ids.is_empty() {
            return false;
        }
        
        solitaire::check_game_clear(world, &self.foundation_ids)
    }
}

impl System for GameStateSystem {
    fn name(&self) -> &'static str {
        "GameStateSystem"
    }
    
    fn phase(&self) -> SystemPhase {
        SystemPhase::Update
    }
    
    fn priority(&self) -> SystemPriority {
        SystemPriority::new(100)  // 低い優先度で実行（他のシステムの後）
    }
    
    fn run(&mut self, world: &mut World, resources: &mut ResourceManager, _delta_time: f32) -> Result<(), JsValue> {
        // ファウンデーションのIDを見つける（初回のみ）
        self.find_foundation_ids(world);
        
        // 現在のゲーム状態を取得
        let game_state = match resources.get::<GameState>() {
            Some(state) => *state,
            None => return Ok(()),  // ゲーム状態がなければ何もしない
        };
        
        // 状態に応じた処理
        match game_state {
            GameState::Title => {
                // タイトル画面の処理
                // 実際のゲームでは、ここでスタート画面の表示などを行う
            },
            GameState::Playing => {
                // プレイ中の処理
                
                // ゲームクリアのチェック
                if self.check_game_clear(world) {
                    // ゲームクリア状態に移行
                    info!("🎉 ゲームクリア！おめでとう！");
                    if let Some(state) = resources.get_mut::<GameState>() {
                        *state = GameState::Clear;
                    }
                }
            },
            GameState::Paused => {
                // 一時停止中の処理
                // 実際のゲームでは、ここで一時停止画面の表示などを行う
            },
            GameState::GameOver => {
                // ゲームオーバーの処理
                // 実際のゲームでは、ここでゲームオーバー画面の表示などを行う
            },
            GameState::Clear => {
                // クリア画面の処理
                // 実際のゲームでは、ここでクリア画面の表示などを行う
            },
        }
        
        Ok(())
    }
}

/// ゲーム状態を変更する関数
pub fn change_game_state(resources: &mut ResourceManager, new_state: GameState) {
    if let Some(state) = resources.get_mut::<GameState>() {
        debug!("ゲーム状態を変更: {:?} -> {:?}", *state, new_state);
        *state = new_state;
    }
}

/// プレイ状態に移行する関数
pub fn start_game(resources: &mut ResourceManager) {
    change_game_state(resources, GameState::Playing);
    info!("🎮 ゲームを開始しました！");
}

/// 一時停止状態に移行する関数
pub fn pause_game(resources: &mut ResourceManager) {
    change_game_state(resources, GameState::Paused);
    info!("⏸️ ゲームを一時停止しました");
}

/// ゲームを再開する関数
pub fn resume_game(resources: &mut ResourceManager) {
    change_game_state(resources, GameState::Playing);
    info!("▶️ ゲームを再開しました");
}

/// ゲームオーバー状態に移行する関数
pub fn game_over(resources: &mut ResourceManager) {
    change_game_state(resources, GameState::GameOver);
    info!("💀 ゲームオーバー");
}

/// タイトル画面に戻る関数
pub fn return_to_title(resources: &mut ResourceManager) {
    change_game_state(resources, GameState::Title);
    info!("🏠 タイトル画面に戻りました");
} 