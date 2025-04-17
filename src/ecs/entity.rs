use std::collections::HashSet;
use wasm_bindgen::prelude::*;
use crate::constants::MAX_ENTITIES;

/// エンティティIDの型定義
/// エンティティを一意に識別するための数値
pub type EntityId = usize;

/// エンティティマネージャー
/// ゲーム内のエンティティの作成、削除、管理を担当する
pub struct EntityManager {
    // 次に割り当てるエンティティID
    next_entity_id: EntityId,
    
    // 現在アクティブなエンティティのIDセット
    active_entities: HashSet<EntityId>,
    
    // 削除予定のエンティティのIDセット
    // 次のフレーム更新時に実際に削除される
    entities_to_remove: HashSet<EntityId>,
}

impl EntityManager {
    /// 新しいエンティティマネージャーを作成
    pub fn new() -> Self {
        Self {
            next_entity_id: 0,
            active_entities: HashSet::new(),
            entities_to_remove: HashSet::new(),
        }
    }
    
    /// 新しいエンティティを作成し、そのIDを返す
    pub fn create_entity(&mut self) -> Result<EntityId, JsValue> {
        // エンティティの最大数をチェック
        if self.active_entities.len() >= MAX_ENTITIES {
            return Err(JsValue::from_str(&format!(
                "エンティティの最大数（{}）に達しました！😱",
                MAX_ENTITIES
            )));
        }
        
        // 新しいエンティティIDを割り当て
        let entity_id = self.next_entity_id;
        self.next_entity_id += 1;
        
        // アクティブなエンティティのセットに追加
        self.active_entities.insert(entity_id);
        
        Ok(entity_id)
    }
    
    /// エンティティを削除予定としてマーク
    /// 実際の削除は次のupdate()呼び出し時に行われる
    pub fn mark_entity_for_removal(&mut self, entity_id: EntityId) {
        if self.active_entities.contains(&entity_id) {
            self.entities_to_remove.insert(entity_id);
        }
    }
    
    /// 削除予定としてマークされたエンティティを実際に削除
    pub fn update(&mut self) {
        // 削除予定のエンティティをアクティブなエンティティから削除
        for entity_id in &self.entities_to_remove {
            self.active_entities.remove(entity_id);
        }
        
        // 削除予定リストをクリア
        self.entities_to_remove.clear();
    }
    
    /// 指定したエンティティがアクティブかどうかをチェック
    pub fn is_entity_active(&self, entity_id: EntityId) -> bool {
        self.active_entities.contains(&entity_id)
    }
    
    /// 現在アクティブなエンティティのIDのイテレータを返す
    pub fn active_entities(&self) -> impl Iterator<Item = &EntityId> {
        self.active_entities.iter()
    }
    
    /// アクティブなエンティティの数を返す
    pub fn entity_count(&self) -> usize {
        self.active_entities.len()
    }
    
    /// 全てのエンティティを削除
    pub fn clear_all_entities(&mut self) {
        self.active_entities.clear();
        self.entities_to_remove.clear();
    }
} 