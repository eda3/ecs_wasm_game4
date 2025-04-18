use wasm_bindgen::prelude::*;
use crate::ecs::entity::{EntityId, EntityManager};
use crate::ecs::component::{Component, ComponentManager};
use crate::ecs::system::{System, SystemManager};
use crate::ecs::resources::ResourceManager;

/// World構造体
/// エンティティ、コンポーネント、システム、リソースを統合管理する
pub struct World {
    // エンティティを管理
    entity_manager: EntityManager,
    
    // コンポーネントを管理
    component_manager: ComponentManager,
    
    // 新しく作成されたエンティティのID
    created_entities: Vec<EntityId>,
}

impl World {
    /// 新しいWorldを作成
    pub fn new() -> Self {
        Self {
            entity_manager: EntityManager::new(),
            component_manager: ComponentManager::new(),
            created_entities: Vec::new(),
        }
    }
    
    //
    // エンティティ関連のメソッド
    //
    
    /// 新しいエンティティを作成し、IDを返す
    pub fn create_entity(&mut self) -> Result<EntityId, JsValue> {
        let entity_id = self.entity_manager.create_entity()?;
        self.created_entities.push(entity_id);
        Ok(entity_id)
    }
    
    /// エンティティを削除
    pub fn remove_entity(&mut self, entity_id: EntityId) {
        // エンティティを削除予定としてマーク
        self.entity_manager.mark_entity_for_removal(entity_id);
        
        // 関連するコンポーネントを全て削除
        self.component_manager.remove_entity(&entity_id);
    }
    
    /// エンティティが存在するかチェック
    pub fn entity_exists(&self, entity_id: EntityId) -> bool {
        self.entity_manager.is_entity_active(entity_id)
    }
    
    /// アクティブなエンティティの数を取得
    pub fn entity_count(&self) -> usize {
        self.entity_manager.entity_count()
    }
    
    /// アクティブなエンティティのIDのベクターを取得
    pub fn get_all_entities(&self) -> Vec<EntityId> {
        self.entity_manager.active_entities().copied().collect()
    }
    
    //
    // コンポーネント関連のメソッド
    //
    
    /// エンティティにコンポーネントを追加
    pub fn add_component<T: Component>(&mut self, entity_id: EntityId, component: T) -> Result<(), JsValue> {
        if !self.entity_exists(entity_id) {
            return Err(JsValue::from_str(&format!(
                "エンティティID: {} は存在しません！",
                entity_id
            )));
        }
        
        self.component_manager.add_component(entity_id, component);
        Ok(())
    }
    
    /// エンティティからコンポーネントを取得
    pub fn get_component<T: Component>(&self, entity_id: EntityId) -> Option<&T> {
        if !self.entity_exists(entity_id) {
            return None;
        }
        
        self.component_manager.get_component(&entity_id)
    }
    
    /// エンティティからコンポーネントを可変で取得
    pub fn get_component_mut<T: Component>(&mut self, entity_id: EntityId) -> Option<&mut T> {
        if !self.entity_exists(entity_id) {
            return None;
        }
        
        self.component_manager.get_component_mut(&entity_id)
    }
    
    /// エンティティからコンポーネントを削除
    pub fn remove_component<T: Component>(&mut self, entity_id: EntityId) -> Option<T> {
        if !self.entity_exists(entity_id) {
            return None;
        }
        
        self.component_manager.remove_component(&entity_id)
    }
    
    /// エンティティが特定のコンポーネントを持っているかチェック
    pub fn has_component<T: Component>(&self, entity_id: EntityId) -> bool {
        if !self.entity_exists(entity_id) {
            return false;
        }
        
        self.component_manager.has_component::<T>(&entity_id)
    }
    
    /// 特定のコンポーネントを持つ全てのエンティティIDを取得
    pub fn get_entities_with_component<T: Component>(&self) -> Vec<EntityId> {
        self.component_manager.entities_with_component::<T>()
    }
    
    //
    // 世界の更新
    //
    
    /// ワールドの状態を更新
    /// 削除予定のエンティティを実際に削除する
    pub fn update(&mut self) {
        self.entity_manager.update();
        self.created_entities.clear();
    }
    
    /// システムを実行
    pub fn run_systems(
        &mut self,
        system_manager: &mut SystemManager,
        resource_manager: &mut ResourceManager,
        delta_time: f32,
    ) -> Result<(), JsValue> {
        // 全てのシステムを実行
        system_manager.run_systems(self, resource_manager, delta_time)?;
        
        // ワールドの状態を更新
        self.update();
        
        Ok(())
    }
    
    /// 全てのエンティティとコンポーネントをクリア
    pub fn clear(&mut self) {
        self.entity_manager.clear_all_entities();
        self.component_manager.clear();
        self.created_entities.clear();
    }
} 