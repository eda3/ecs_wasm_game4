use wasm_bindgen::prelude::*;
use crate::ecs::world::World;
use crate::ecs::system::{System, SystemPhase, SystemPriority};
use crate::ecs::resources::ResourceManager;
use crate::ecs::component::{Transform, Renderable, CardInfo, StackContainer, StackType};
use crate::constants::STACK_OFFSET_Y;
use log::error;

/// レンダリングシステム
/// ゲーム世界の状態を描画する責任を持つ
pub struct RenderSystem {
    // 将来的な拡張のためのフィールドを追加可能
}

impl RenderSystem {
    /// 新しいレンダリングシステムを作成
    pub fn new() -> Self {
        Self {}
    }
    
    /// スタックコンテナのカードの位置を更新
    fn update_stack_positions(&self, world: &mut World) -> Result<(), JsValue> {
        // StackContainerコンポーネントを持つエンティティを取得
        let entities_with_stack = world.get_entities_with_component::<StackContainer>();
        
        for &stack_entity_id in &entities_with_stack {
            // スタックの情報を取得
            if let Some(stack) = world.get_component::<StackContainer>(stack_entity_id) {
                // スタックの位置を取得
                if let Some(stack_transform) = world.get_component::<Transform>(stack_entity_id) {
                    let base_x = stack_transform.position.x;
                    let base_y = stack_transform.position.y;
                    
                    // スタック内のカードIDをコピーして所有権問題を回避
                    let card_ids = stack.cards.clone();
                    
                    // スタックのタイプに応じて位置を更新
                    match stack.stack_type {
                        StackType::Tableau { .. } => {
                            // タブローの場合、カードを縦に少しずつ重ねて表示
                            for (i, &card_id) in card_ids.iter().enumerate() {
                                if let Some(card_transform) = world.get_component_mut::<Transform>(card_id) {
                                    let y_offset = i as f64 * STACK_OFFSET_Y;
                                    card_transform.position.x = base_x;
                                    card_transform.position.y = base_y + y_offset;
                                    card_transform.z_index = i as i32;
                                }
                            }
                        },
                        StackType::Foundation { .. } | StackType::Stock | StackType::Waste => {
                            // ファウンデーション、ストック、ウェイストの場合、カードを完全に重ねて表示
                            for (i, &card_id) in card_ids.iter().enumerate() {
                                if let Some(card_transform) = world.get_component_mut::<Transform>(card_id) {
                                    card_transform.position.x = base_x;
                                    card_transform.position.y = base_y;
                                    card_transform.z_index = i as i32;
                                }
                            }
                        },
                        StackType::Hand => {
                            // 手札（ドラッグ中）の場合、特に何もしない
                            // ドラッグシステムがこれを処理する
                        },
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// 描画のために必要な視覚的な更新を行う
    fn update_visual_state(&self, world: &mut World) -> Result<(), JsValue> {
        // ドラッグ中のカードの不透明度を調整するなど
        
        // カードが表向きかどうかに応じてドラッグ可能かを更新
        let entities_with_card = world.get_entities_with_component::<CardInfo>();
        
        for &card_id in &entities_with_card {
            if let Some(_card_info) = world.get_component::<CardInfo>(card_id) {
                if let Some(_renderable) = world.get_component_mut::<Renderable>(card_id) {
                    // カードが表向きかどうかで描画タイプを調整
                    // （実際には何もする必要がないが、将来的な拡張のため）
                }
            }
        }
        
        Ok(())
    }
}

impl System for RenderSystem {
    fn name(&self) -> &'static str {
        "RenderSystem"
    }
    
    fn phase(&self) -> SystemPhase {
        SystemPhase::Render  // 描画フェーズで実行
    }
    
    fn priority(&self) -> SystemPriority {
        SystemPriority::new(0)  // 描画フェーズ内で最初に実行
    }
    
    fn run(&mut self, world: &mut World, _resources: &mut ResourceManager, _delta_time: f32) -> Result<(), JsValue> {
        // スタックコンテナ内のカードの位置を更新
        if let Err(e) = self.update_stack_positions(world) {
            error!("スタック位置の更新中にエラーが発生しました: {:?}", e);
        }
        
        // 視覚的な状態を更新
        if let Err(e) = self.update_visual_state(world) {
            error!("視覚的状態の更新中にエラーが発生しました: {:?}", e);
        }
        
        Ok(())
    }
} 