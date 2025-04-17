use wasm_bindgen::prelude::*;
use crate::ecs::world::World;
use crate::ecs::resources::ResourceManager;
use std::cmp::Ordering;

/// システムフェーズ
/// システムの実行順序を決定するためのフェーズ
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SystemPhase {
    Input,       // 入力処理
    PreUpdate,   // メイン更新前
    Update,      // メイン更新
    PostUpdate,  // メイン更新後
    Render,      // 描画
}

/// システム優先度
/// 同じフェーズ内でのシステムの実行順序を決定する
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemPriority(pub i32);

impl SystemPriority {
    pub fn new(priority: i32) -> Self {
        Self(priority)
    }
}

impl PartialOrd for SystemPriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SystemPriority {
    fn cmp(&self, other: &Self) -> Ordering {
        // 値が小さいほど優先度が高い
        self.0.cmp(&other.0)
    }
}

/// システムトレイト
/// 全てのシステムが実装する必要があるトレイト
pub trait System {
    /// システムの名前を返す
    /// デバッグやプロファイリング用
    fn name(&self) -> &'static str;
    
    /// システムのフェーズを返す
    /// どのタイミングでシステムを実行するかを決定する
    fn phase(&self) -> SystemPhase {
        SystemPhase::Update
    }
    
    /// システムの優先度を返す
    /// 同じフェーズ内での実行順序を決定する
    fn priority(&self) -> SystemPriority {
        SystemPriority(0)
    }
    
    /// システムの実行メソッド
    /// world: エンティティとコンポーネントを含むゲームの世界
    /// resources: グローバルなリソース（例：入力状態、時間など）
    /// delta_time: 前回のフレームからの経過時間（秒）
    fn run(&mut self, world: &mut World, resources: &mut ResourceManager, delta_time: f32) -> Result<(), JsValue>;
}

/// システムマネージャー
/// 複数のシステムを管理し、適切な順序で実行する
pub struct SystemManager {
    systems: Vec<Box<dyn System>>,
}

impl SystemManager {
    /// 新しいシステムマネージャーを作成
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }
    
    /// システムを追加
    pub fn add_system<S: System + 'static>(&mut self, system: S) {
        self.systems.push(Box::new(system));
    }
    
    /// 全てのシステムをソートして適切な順序で実行
    pub fn run_systems(&mut self, world: &mut World, resources: &mut ResourceManager, delta_time: f32) -> Result<(), JsValue> {
        // フェーズと優先度でシステムをソート
        self.systems.sort_by(|a, b| {
            let phase_cmp = a.phase().cmp(&b.phase());
            if phase_cmp == Ordering::Equal {
                a.priority().cmp(&b.priority())
            } else {
                phase_cmp
            }
        });
        
        // 全てのシステムを実行
        for system in &mut self.systems {
            system.run(world, resources, delta_time)?;
        }
        
        Ok(())
    }
    
    /// 特定のフェーズのシステムのみを実行
    pub fn run_systems_for_phase(
        &mut self,
        phase: SystemPhase,
        world: &mut World,
        resources: &mut ResourceManager,
        delta_time: f32,
    ) -> Result<(), JsValue> {
        // フェーズと優先度でシステムをソート
        self.systems.sort_by(|a, b| {
            let phase_cmp = a.phase().cmp(&b.phase());
            if phase_cmp == Ordering::Equal {
                a.priority().cmp(&b.priority())
            } else {
                phase_cmp
            }
        });
        
        // 指定されたフェーズのシステムのみを実行
        for system in &mut self.systems {
            if system.phase() == phase {
                system.run(world, resources, delta_time)?;
            }
        }
        
        Ok(())
    }
    
    /// 全てのシステムをクリア
    pub fn clear(&mut self) {
        self.systems.clear();
    }
} 