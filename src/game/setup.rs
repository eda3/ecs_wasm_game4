use wasm_bindgen::prelude::*;
use crate::ecs::world::World;
use crate::ecs::system::SystemManager;
use crate::ecs::resources::{ResourceManager, TimeInfo, InputState, GameState, NetworkState};
use crate::constants::TARGET_FPS;
use crate::game::solitaire;
use crate::render::systems::RenderSystem;
use crate::input::systems::{InputSystem, DragSystem};
use crate::game::state::GameStateSystem;
use log::{info, error};

/// ゲームの初期化を行う関数
pub fn setup_game(
    world: &mut World,
    system_manager: &mut SystemManager,
    resource_manager: &mut ResourceManager,
) -> Result<(), JsValue> {
    info!("🎮 ゲームをセットアップ中...");
    
    // リソースを初期化
    setup_resources(resource_manager);
    
    // システムを初期化
    setup_systems(system_manager);
    
    // ゲーム世界を初期化
    setup_world(world)?;
    
    info!("✅ ゲームのセットアップが完了しました！");
    Ok(())
}

/// リソースのセットアップ
fn setup_resources(resource_manager: &mut ResourceManager) {
    info!("📦 リソースを初期化中...");
    
    // 時間情報を初期化
    let time_info = TimeInfo::new(TARGET_FPS);
    resource_manager.add(time_info);
    
    // 入力状態を初期化
    let input_state = InputState::new();
    resource_manager.add(input_state);
    
    // ゲーム状態を初期化
    resource_manager.add(GameState::Title);
    
    // ネットワーク状態を初期化
    let network_state = NetworkState::new();
    resource_manager.add(network_state);
}

/// システムのセットアップ
fn setup_systems(system_manager: &mut SystemManager) {
    info!("⚙️ システムを初期化中...");
    
    // 入力システムを追加
    system_manager.add_system(InputSystem::new());
    
    // ドラッグシステムを追加
    system_manager.add_system(DragSystem::new());
    
    // ゲーム状態システムを追加
    system_manager.add_system(GameStateSystem::new());
    
    // レンダリングシステムを追加
    system_manager.add_system(RenderSystem::new());
}

/// ゲーム世界のセットアップ
fn setup_world(world: &mut World) -> Result<(), JsValue> {
    info!("🌍 ゲーム世界を初期化中...");
    
    // ソリティアボードをセットアップ
    solitaire::setup_solitaire_board(world)?;
    
    Ok(())
} 