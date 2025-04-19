use wasm_bindgen::prelude::*;
use crate::ecs::world::World;
use crate::ecs::system::{System, SystemPhase, SystemPriority};
use crate::ecs::resources::{ResourceManager, InputState};
use crate::ecs::component::{Transform, Draggable, Clickable, StackContainer, StackType, Droppable, Renderable};
use crate::ecs::entity::EntityId;
use crate::input::input_handler::InputHandler;
use crate::utils::Vec2;
use crate::constants::{DRAG_OPACITY};
use log::debug;
/// 入力処理システム
/// マウスやキーボードの入力を処理し、ゲーム状態を更新する
pub struct InputSystem {
    // 前回のマウス座標
    last_mouse_position: Vec2,
    
    // クリックされたエンティティ
    clicked_entity: Option<EntityId>,
}
impl InputSystem {
    /// 新しい入力システムを作成
    pub fn new() -> Self {
        Self {
            last_mouse_position: Vec2::zero(),
            clicked_entity: None,
        }
    }
    
    /// クリックされたエンティティを処理
    fn process_click(
        &mut self,
        world: &mut World,
        entity_id: EntityId,
    ) -> Result<(), JsValue> {
        // クリック可能コンポーネントを持つかチェック
        if let Some(clickable) = world.get_component_mut::<Clickable>(entity_id) {
            // クリックされたことをマーク
            clickable.was_clicked = true;
            debug!("🖱️ エンティティ {} がクリックされました", entity_id);
            
            // クリックハンドラーのタイプに応じて処理
            match &clickable.click_handler {
                crate::ecs::component::ClickHandlerType::FlipCard => {
                    // カードをめくる
                    if let Some(card_info) = world.get_component_mut::<crate::ecs::component::CardInfo>(entity_id) {
                        card_info.face_up = !card_info.face_up;
                        debug!("🃏 カード {} を{}", entity_id, if card_info.face_up { "表向き" } else { "裏向き" });
                    }
                },
                crate::ecs::component::ClickHandlerType::DrawFromStock => {
                    // ストックからカードを引く処理
                    let (stock_id, waste_id) = self.find_stock_and_waste(world)?;
                    crate::game::solitaire::draw_from_stock(world, stock_id, waste_id)?;
                },
                crate::ecs::component::ClickHandlerType::DrawFromWaste => {
                    // ウェイストからカードを引く処理
                    // 実際の実装はもっと複雑になるが、ここではシンプルに
                    debug!("🃏 ウェイストからカードを引く処理");
                },
                crate::ecs::component::ClickHandlerType::DrawFromTableau { column } => {
                    // タブローからカードを引く処理
                    debug!("🃏 タブロー列 {} からカードを引く処理", column);
                },
                crate::ecs::component::ClickHandlerType::DrawFromFoundation { stack } => {
                    // ファウンデーションからカードを引く処理
                    debug!("🃏 ファウンデーションスタック {} からカードを引く処理", stack);
                },
                crate::ecs::component::ClickHandlerType::Custom => {
                    // カスタム処理（必要に応じて実装）
                },
            }
        }
        
        Ok(())
    }
    
    /// ストックとウェイストのエンティティIDを検索
    fn find_stock_and_waste(&self, world: &World) -> Result<(EntityId, EntityId), JsValue> {
        let mut stock_id = None;
        let mut waste_id = None;
        
        // StackContainerコンポーネントを持つエンティティを探索
        let entities_with_stack = world.get_entities_with_component::<StackContainer>();
        
        for &entity_id in &entities_with_stack {
            if let Some(stack) = world.get_component::<StackContainer>(entity_id) {
                match stack.stack_type {
                    StackType::Stock => {
                        stock_id = Some(entity_id);
                    },
                    StackType::Waste => {
                        waste_id = Some(entity_id);
                    },
                    _ => {},
                }
            }
        }
        
        let stock_id = stock_id.ok_or_else(|| JsValue::from_str("ストックエンティティが見つかりません"))?;
        let waste_id = waste_id.ok_or_else(|| JsValue::from_str("ウェイストエンティティが見つかりません"))?;
        
        Ok((stock_id, waste_id))
    }
}
impl System for InputSystem {
    fn name(&self) -> &'static str {
        "InputSystem"
    }
    
    fn phase(&self) -> SystemPhase {
        SystemPhase::Input  // 入力フェーズで実行
    }
    
    fn priority(&self) -> SystemPriority {
        SystemPriority::new(0)  // 入力フェーズ内で最初に実行
    }
    
    fn run(
        &mut self,
        world: &mut World,
        _resources: &mut ResourceManager,
        _delta_time: f32,
    ) -> Result<(), JsValue> {
        // 入力状態を取得
        let input_state = match _resources.get::<InputState>() {
            Some(state) => state,  // 参照を使用
            None => return Ok(()),  // 入力状態がなければ何もしない
        };
        
        // マウスがクリックされた瞬間を検出
        if input_state.is_mouse_down && !input_state.mouse_buttons[0] {
            // エンティティを探す
            if let Some(entity_id) = InputHandler::get_entity_at_position(
                world,
                input_state.mouse_position,
            ) {
                self.clicked_entity = Some(entity_id);
                self.process_click(world, entity_id)?;
            }
        }
        
        // クリック状態をリセット
        if !input_state.is_mouse_down && self.clicked_entity.is_some() {
            if let Some(entity_id) = self.clicked_entity {
                if let Some(clickable) = world.get_component_mut::<Clickable>(entity_id) {
                    clickable.was_clicked = false;
                }
            }
            self.clicked_entity = None;
        }
        
        // マウス位置を記録
        self.last_mouse_position = input_state.mouse_position;
        
        Ok(())
    }
}
/// ドラッグ処理システム
/// ドラッグ可能なエンティティのドラッグ操作を処理する
pub struct DragSystem {
    // 現在ドラッグ中のエンティティ
    dragged_entity: Option<EntityId>,
    
    // ドラッグ開始時のマウス位置
    drag_start_position: Vec2,
    
    // ドラッグ操作が開始されたかどうか
    drag_started: bool,
    
    // 前回のマウス位置
    last_mouse_pos: Vec2,
    
    // 前回のフレームで左ボタンが押されていたか
    left_button_pressed_prev: bool,
    
    // ドラッグ中のエンティティの元のZ-index
    original_z_index: i32,
}
impl DragSystem {
    /// 新しいドラッグシステムを作成
    pub fn new() -> Self {
        Self {
            dragged_entity: None,
            drag_start_position: Vec2::zero(),
            drag_started: false,
            last_mouse_pos: Vec2::zero(),
            left_button_pressed_prev: false,  // 明示的にfalseで初期化
            original_z_index: 0,
        }
    }
    
    /// ドラッグ可能なエンティティを探す
    fn find_draggable_entity(&self, world: &World, position: Vec2) -> Option<EntityId> {
        // 座標にあるエンティティを取得
        let entity_id = InputHandler::get_entity_at_position(world, position)?;
        
        // ドラッグ可能かチェック
        if world.has_component::<Draggable>(entity_id) {
            Some(entity_id)
        } else {
            None
        }
    }
    
    /// ドラッグを開始
    fn start_drag(&mut self, world: &mut World, entity_id: EntityId, mouse_position: Vec2) -> Result<(), JsValue> {
        debug!("🚀 エンティティ {} のドラッグ開始処理を実行中...", entity_id);
        debug!("🖱️ マウス位置=({:.1}, {:.1})", mouse_position.x, mouse_position.y);
        
        // 必要な情報を先に取得
        let transform_position;
        let transform_z_index;
        
        // 1. エンティティの現在位置を先に取得
        {
            if let Some(transform) = world.get_component::<crate::ecs::component::Transform>(entity_id) {
                transform_position = transform.position.clone(); // cloneを明示的に呼び出す
                transform_z_index = transform.z_index;
                debug!("📍 エンティティ {} の位置: ({:.1}, {:.1}), Z-index: {}", 
                    entity_id, transform_position.x, transform_position.y, transform_z_index);
            } else {
                // Transformがなければ処理を中止
                debug!("❌ エラー: エンティティ {} にTransformコンポーネントがありません", entity_id);
                return Ok(());
            }
        }
        
        // 2. ドラッグオフセットを計算
        let drag_offset = Vec2::new(
            mouse_position.x - transform_position.x,
            mouse_position.y - transform_position.y,
        );
        debug!("📏 ドラッグオフセット: ({:.1}, {:.1})", drag_offset.x, drag_offset.y);
        
        // 3. ドラッグ可能コンポーネントを更新
        let drag_component_updated = if let Some(draggable) = world.get_component_mut::<Draggable>(entity_id) {
            debug!("🔄 ドラッグ状態（更新前）: is_dragging={}, original_z_index={}", 
                draggable.is_dragging, draggable.original_z_index);
                
            draggable.is_dragging = true;
            draggable.original_position = transform_position;
            draggable.original_z_index = transform_z_index;
            draggable.drag_offset = drag_offset;
            
            debug!("✅ ドラッグ状態（更新後）: is_dragging=true, original_position=({:.1}, {:.1}), original_z_index={}, drag_offset=({:.1}, {:.1})", 
                draggable.original_position.x, draggable.original_position.y, 
                draggable.original_z_index, draggable.drag_offset.x, draggable.drag_offset.y);
            true
        } else {
            debug!("❌ エラー: エンティティ {} にDraggableコンポーネントがありません", entity_id);
            false
        };
        
        if !drag_component_updated {
            debug!("❌ Draggableコンポーネントの更新に失敗しました。処理を中止します。");
            return Ok(());
        }
        
        // 4. レンダラブルコンポーネントの不透明度を下げる
        let opacity_updated = if let Some(renderable) = world.get_component_mut::<crate::ecs::component::Renderable>(entity_id) {
            debug!("🎨 元の不透明度: {}", renderable.opacity);
            renderable.opacity = crate::constants::DRAG_OPACITY;
            debug!("🎨 新しい不透明度: {} に設定しました", renderable.opacity);
            true
        } else {
            debug!("❌ エラー: エンティティ {} にRenderableコンポーネントがありません", entity_id);
            false
        };
        
        if !opacity_updated {
            debug!("⚠️ 警告: 不透明度の更新に失敗しましたが、処理は続行します");
        }
        
        // 5. カードがタブローのスタックにある場合、そのカード以降のカードも一緒にドラッグ
        let mut cards_to_drag = Vec::new();
        
        // カードがどのスタックに属しているか確認
        let stacks = world.get_entities_with_component::<crate::ecs::component::StackContainer>();
        debug!("📦 スタックコンテナの総数: {}", stacks.len());
        
        let mut found_stack = false;
        for &stack_id in &stacks {
            if let Some(stack) = world.get_component::<crate::ecs::component::StackContainer>(stack_id) {
                // カードがこのスタックに含まれているか確認
                if let Some(card_index) = stack.cards.iter().position(|&card| card == entity_id) {
                    debug!("📦 カードがスタック {} の {}番目に見つかりました。スタックタイプ: {:?}", 
                        stack_id, card_index, stack.stack_type);
                    found_stack = true;
                    
                    // タブローのスタックのみ、カード以降も一緒にドラッグ
                    if let crate::ecs::component::StackType::Tableau { .. } = stack.stack_type {
                        debug!("📦 これはタブローのスタックなので、このカード以降も一緒にドラッグします");
                        cards_to_drag = stack.cards_from_index(card_index);
                        debug!("📦 一緒にドラッグするカード: {} 枚 {:?}", cards_to_drag.len(), cards_to_drag);
                        
                        // 一番上のカード以外の不透明度も下げる
                        if cards_to_drag.len() > 1 {
                            debug!("📦 複数のカードをドラッグします: {} 枚", cards_to_drag.len());
                            
                            // カードの詳細情報を出力
                            for (i, &card_id) in cards_to_drag.iter().enumerate() {
                                if let Some(card_info) = world.get_component::<crate::ecs::component::CardInfo>(card_id) {
                                    debug!("🃏 カード {}: ID={}, スート={}, ランク={}, 表向き={}", 
                                        i, card_id, card_info.suit, card_info.rank, card_info.face_up);
                                }
                            }
                            
                            for (i, &card_id) in cards_to_drag.iter().enumerate().skip(1) {
                                debug!("📦 追加カード {} の処理中...", card_id);
                                
                                // 1. 不透明度を下げる
                                if let Some(card_renderable) = world.get_component_mut::<crate::ecs::component::Renderable>(card_id) {
                                    debug!("🎨 カード {} の不透明度を {} に設定します", card_id, crate::constants::DRAG_OPACITY);
                                    card_renderable.opacity = crate::constants::DRAG_OPACITY;
                                } else {
                                    debug!("❌ カード {} にRenderableコンポーネントがありません", card_id);
                                }
                                
                                // 2. 必要なデータを先に取得
                                let position;
                                let z_index;
                                {
                                    if let Some(card_transform) = world.get_component::<crate::ecs::component::Transform>(card_id) {
                                        position = card_transform.position.clone();
                                        z_index = card_transform.z_index;
                                        debug!("📍 カード {} の位置: ({:.1}, {:.1}), Z-index: {}", 
                                            card_id, position.x, position.y, z_index);
                                    } else {
                                        debug!("❌ カード {} にTransformコンポーネントがありません", card_id);
                                        continue;
                                    }
                                }
                                
                                // 3. Draggableコンポーネントを更新
                                if let Some(card_draggable) = world.get_component_mut::<crate::ecs::component::Draggable>(card_id) {
                                    card_draggable.original_position = position;
                                    card_draggable.original_z_index = z_index;
                                    // 実際にドラッグされてるようにフラグを設定
                                    card_draggable.is_dragging = true;
                                    debug!("✅ カード {} のドラッグ状態を更新しました", card_id);
                                } else {
                                    debug!("❌ カード {} にDraggableコンポーネントがありません", card_id);
                                }
                                
                                // 4. 別のスコープでTransformコンポーネントを再度取得して更新
                                if let Some(card_transform) = world.get_component_mut::<crate::ecs::component::Transform>(card_id) {
                                    // Z-indexを調整して重なる順序を維持
                                    let new_z_index = 1000 + i as i32;
                                    debug!("📍 カード {} のZ-indexを {} から {} に更新します", card_id, card_transform.z_index, new_z_index);
                                    card_transform.z_index = new_z_index;
                                }
                            }
                        }
                    } else {
                        debug!("📦 これはタブロー以外のスタック（{:?}）なので、このカードのみドラッグします", stack.stack_type);
                    }
                    break;
                }
            }
        }
        
        if !found_stack {
            debug!("⚠️ カードがどのスタックにも属していません");
        }
        
        // 6. ドラッグ中のエンティティを記録
        self.dragged_entity = Some(entity_id);
        self.drag_start_position = mouse_position;
        self.drag_started = true;
        
        debug!("✨ エンティティ {} のドラッグを開始しました！一緒にドラッグするカード: {}枚", entity_id, cards_to_drag.len());
        
        // 現在のドラッグ状態を確認
        debug!("📊 ドラッグ状態: dragged_entity={:?}, drag_started={}, drag_start_position=({:.1}, {:.1})", 
            self.dragged_entity, self.drag_started, self.drag_start_position.x, self.drag_start_position.y);
        
        Ok(())
    }
    
    /// ドラッグ中の更新
    fn update_drag(&mut self, world: &mut World, entity_id: EntityId, mouse_position: Vec2) -> Result<(), JsValue> {
        // ドラッグオフセットを取得
        let drag_offset = if let Some(draggable) = world.get_component::<Draggable>(entity_id) {
            draggable.drag_offset
        } else {
            Vec2::zero() // デフォルト値
        };
        
        // エンティティの位置を更新
        if let Some(transform) = world.get_component_mut::<Transform>(entity_id) {
            transform.position.x = mouse_position.x - drag_offset.x;
            transform.position.y = mouse_position.y - drag_offset.y;
            
            // Z-indexを大きくして最前面に表示
            transform.z_index = 1000;
        }
        
        // スタック内の追加カードも移動
        let mut cards_to_update = Vec::new();
        
        // カードがどのスタックに属しているか確認
        let stacks = world.get_entities_with_component::<StackContainer>();
        for &stack_id in &stacks {
            if let Some(stack) = world.get_component::<StackContainer>(stack_id) {
                // カードがこのスタックに含まれているか確認
                if let Some(card_index) = stack.cards.iter().position(|&card| card == entity_id) {
                    // タブローのスタックのみ、カード以降も一緒にドラッグ
                    if let crate::ecs::component::StackType::Tableau { .. } = stack.stack_type {
                        cards_to_update = stack.cards_from_index(card_index + 1);
                    }
                    break;
                }
            }
        }
        
        // 追加カードの位置も更新
        let base_x = mouse_position.x - drag_offset.x;
        let base_y = mouse_position.y - drag_offset.y;
        
        for (i, &card_id) in cards_to_update.iter().enumerate() {
            if let Some(transform) = world.get_component_mut::<Transform>(card_id) {
                transform.position.x = base_x;
                transform.position.y = base_y + (i as f64 + 1.0) * crate::constants::STACK_OFFSET_Y;
                transform.z_index = 1000 + (i as i32 + 1);
            }
        }
        
        Ok(())
    }
    
    /// ドラッグを終了
    fn end_drag(&self, world: &mut World) -> Result<(), JsValue> {
        if let Some(entity_id) = self.dragged_entity {
            debug!("👆 エンティティ {} のドラッグを終了", entity_id);
            
            if let Some(draggable) = world.get_component_mut::<Draggable>(entity_id) {
                draggable.is_dragging = false;
                
                // 最終位置を記録
                if let Some(transform) = world.get_component::<Transform>(entity_id) {
                    debug!("📍 ドラッグ終了位置: ({:.1}, {:.1})", 
                        transform.position.x, transform.position.y);
                    
                    // z-indexを元に戻す
                    if let Some(mut transform) = world.get_component_mut::<Transform>(entity_id) {
                        transform.z_index = self.original_z_index;
                        debug!("📊 エンティティ {} のz_indexを元に戻しました: 1000 -> {}", 
                            entity_id, self.original_z_index);
                    }
                }
            } else {
                debug!("❌ エンティティ {} には Draggable コンポーネントがありません", entity_id);
            }
            
            // オブジェクトの透明度を元に戻す
            if let Some(mut renderable) = world.get_component_mut::<Renderable>(entity_id) {
                renderable.opacity = 1.0;
                debug!("🔅 エンティティ {} の透明度を元に戻しました: opacity=1.0", entity_id);
            }
        } else {
            debug!("❓ ドラッグを終了しようとしましたが、ドラッグ中のエンティティがありません");
        }
        
        Ok(())
    }
    
    /// ドロップターゲットを見つける
    fn find_drop_target(&self, world: &World, position: Vec2, dragged_entity: EntityId) -> Result<Option<EntityId>, JsValue> {
        // ドロップ可能なエンティティを探す
        let droppable_entities = world.get_entities_with_component::<Droppable>();
        
        let mut potential_target = None;
        let mut highest_z_index = -1;
        
        // すべてのドロップ可能なエンティティをチェック
        for &entity_id in &droppable_entities {
            // 自分自身はスキップ
            if entity_id == dragged_entity {
                continue;
            }
            
            if let Some(transform) = world.get_component::<Transform>(entity_id) {
                if let Some(droppable) = world.get_component::<Droppable>(entity_id) {
                    // ポジションが範囲内かチェック
                    if position.x >= transform.position.x
                        && position.x <= transform.position.x + droppable.width
                        && position.y >= transform.position.y
                        && position.y <= transform.position.y + droppable.height
                    {
                        // Z-indexが高いものを優先
                        if transform.z_index > highest_z_index {
                            highest_z_index = transform.z_index;
                            potential_target = Some(entity_id);
                        }
                    }
                }
            }
        }
        
        Ok(potential_target)
    }
    
    /// ドロップが有効かどうかチェック
    fn is_valid_drop(&self, world: &World, dragged_entity: EntityId, target_entity: EntityId) -> Result<bool, JsValue> {
        // ここでドロップの有効性をチェックするロジックを実装
        // 例: カードがスタックに追加できるか、アイテムが特定のスロットに配置できるかなど
        
        // 現在はシンプルな例として、すべてのドロップを有効とする
        if let Some(_draggable) = world.get_component::<Draggable>(dragged_entity) {
            if let Some(_droppable) = world.get_component::<Droppable>(target_entity) {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// ドロップ先候補をハイライト表示する
    fn highlight_drop_target(&self, world: &mut World, position: &Vec2) -> Result<(), JsValue> {
        // ドラッグ中のエンティティがない場合は何もしない
        let dragged_entity = match self.dragged_entity {
            Some(entity) => entity,
            None => return Ok(()),
        };
        
        debug!("🔍 ドロップ先候補の検索中: ドラッグ中のエンティティ={}, 位置=({:.1}, {:.1})", 
            dragged_entity, position.x, position.y);
        
        // 以前のハイライトをリセット
        let droppable_entities = world.get_entities_with_component::<Droppable>();
        for &entity_id in &droppable_entities {
            if let Some(mut droppable) = world.get_component_mut::<Droppable>(entity_id) {
                if droppable.is_active {
                    debug!("🔄 エンティティ {} のハイライトをリセット", entity_id);
                    droppable.is_active = false;
                }
            }
        }
        
        // ドロップ可能なエンティティを探す
        if let Ok(Some(drop_target)) = self.find_drop_target(world, position.clone(), dragged_entity) {
            debug!("✓ ドロップ先候補を見つけました: エンティティID={}", drop_target);
            
            // ドロップ先が有効かチェック
            if let Ok(is_valid) = self.is_valid_drop(world, dragged_entity, drop_target) {
                if is_valid {
                    // ハイライト表示
                    if let Some(mut droppable) = world.get_component_mut::<Droppable>(drop_target) {
                        debug!("✨ エンティティ {} をハイライト表示", drop_target);
                        droppable.is_active = true;
                    }
                } else {
                    debug!("✗ ドロップ先 {} は無効です", drop_target);
                }
            }
        } else {
            debug!("✗ ドロップ先候補が見つかりませんでした");
        }
        
        Ok(())
    }
    
    /// ドロップ処理を行う
    fn process_drop(&mut self, world: &mut World, dragged_entity: EntityId, drop_target: EntityId) -> Result<(), JsValue> {
        debug!("🎯 エンティティ {} をエンティティ {} の上にドロップ", dragged_entity, drop_target);
        
        // 必要な情報を先に取得
        let mut should_move_card = false;
        let _target_stack: Option<crate::ecs::component::StackContainer> = None;
        let _card_info: Option<crate::ecs::component::CardInfo> = None;
        let _source_stack: Option<EntityId> = None;
        
        // カード情報を取得
        let card_info = if let Some(info) = world.get_component::<crate::ecs::component::CardInfo>(dragged_entity) {
            Some(info.clone())
        } else {
            None
        };
        
        // ドロップ先がスタックコンテナかチェック
        let target_stack_container = if let Some(stack) = world.get_component::<StackContainer>(drop_target) {
            Some(stack.clone())
        } else {
            None
        };
        
        // ドラッグしてるカードがどのスタックから来たかを調べる
        let source_stack_id = {
            let mut found_stack = None;
            let stacks = world.get_entities_with_component::<StackContainer>();
            
            for &stack_id in &stacks {
                if let Some(stack) = world.get_component::<StackContainer>(stack_id) {
                    if stack.cards.contains(&dragged_entity) {
                        found_stack = Some(stack_id);
                        break;
                    }
                }
            }
            
            found_stack
        };
        
        // ドロップが有効かチェック（ソリティアのルールに基づく）
        if let (Some(card_info), Some(target_stack)) = (card_info, target_stack_container) {
            match target_stack.stack_type {
                crate::ecs::component::StackType::Foundation { suit } => {
                    // 組み札のルール: 同じスートで昇順（A, 2, 3, ...）
                    if card_info.suit as usize == suit {
                        let top_card = target_stack.top_card();
                        if let Some(top_id) = top_card {
                            if let Some(top_info) = world.get_component::<crate::ecs::component::CardInfo>(top_id) {
                                // 次のランクなら配置可能
                                should_move_card = card_info.rank == top_info.rank + 1;
                            }
                        } else {
                            // 空のファウンデーションにはAのみ置ける
                            should_move_card = card_info.rank == 0; // A
                        }
                    }
                },
                crate::ecs::component::StackType::Tableau { .. } => {
                    // 場札のルール: 異なる色で降順（K, Q, J, ...）
                    let top_card = target_stack.top_card();
                    if let Some(top_id) = top_card {
                        if let Some(top_info) = world.get_component::<crate::ecs::component::CardInfo>(top_id) {
                            // 色が異なり、降順なら配置可能
                            let is_diff_color = card_info.is_red() != top_info.is_red();
                            should_move_card = is_diff_color && card_info.rank + 1 == top_info.rank;
                        }
                    } else {
                        // 空の場札にはKのみ置ける
                        should_move_card = card_info.rank == 12; // K
                    }
                },
                _ => {} // その他のスタックは特別ルールなし
            }
        }
        
        // カードを移動（ドロップが有効な場合）
        if should_move_card {
            // 元のスタックからカードを取り除く
            if let Some(source_id) = source_stack_id {
                if let Some(source_stack) = world.get_component_mut::<StackContainer>(source_id) {
                    source_stack.remove_card(dragged_entity);
                }
            }
            
            // 新しいスタックにカードを追加
            if let Some(target_stack) = world.get_component_mut::<StackContainer>(drop_target) {
                target_stack.add_card(dragged_entity);
                
                // 1. 先に必要なデータを取得
                let drop_position;
                let cards_count;
                {
                    // スタックの現在のカード数を保存
                    cards_count = target_stack.cards.len();
                    
                    // ここでtarget_stackのスコープ終了
                }
                
                // 2. ドロップ先のTransformコンポーネントから位置情報を取得
                {
                    if let Some(target_transform) = world.get_component::<Transform>(drop_target) {
                        drop_position = target_transform.position.clone();
                    } else {
                        drop_position = Vec2::zero();
                    }
                }
                
                // 3. スタックのカード数に基づいて位置を計算
                let offset_y = cards_count as f64 * crate::constants::STACK_OFFSET_Y;
                
                // 4. ドラッグしたカードのTransformを更新
                if let Some(transform) = world.get_component_mut::<Transform>(dragged_entity) {
                    transform.position = Vec2::new(
                        drop_position.x,
                        drop_position.y + offset_y
                    );
                    transform.z_index = cards_count as i32;
                }
            }
        } else {
            // ドロップが無効なら元の位置に戻す
            if let Some(draggable) = world.get_component::<Draggable>(dragged_entity) {
                let original_position = draggable.original_position;
                let original_z_index = draggable.original_z_index;
                
                if let Some(transform) = world.get_component_mut::<Transform>(dragged_entity) {
                    transform.position = original_position;
                    transform.z_index = original_z_index;
                }
            }
        }
        
        // ドラッグ状態をリセット
        if let Some(draggable) = world.get_component_mut::<Draggable>(dragged_entity) {
            draggable.is_dragging = false;
        }
        
        // レンダラブルコンポーネントの不透明度を元に戻す
        if let Some(renderable) = world.get_component_mut::<Renderable>(dragged_entity) {
            renderable.opacity = 1.0;
        }
        
        Ok(())
    }
    
    /// 複数カードのドロップを処理
    fn process_multi_card_drop(
        &mut self, 
        world: &mut World, 
        dragged_cards: Vec<EntityId>, 
        target_id: EntityId
    ) -> Result<(), JsValue> {
        debug!("🎯 複数のカード（{}枚）をエンティティ {} の上にドロップ", dragged_cards.len(), target_id);
        
        if dragged_cards.is_empty() {
            return Ok(());
        }
        
        // メインカード（最初にドラッグしたカード）
        let main_card_id = dragged_cards[0];
        
        // カード情報を取得
        let card_info = if let Some(info) = world.get_component::<crate::ecs::component::CardInfo>(main_card_id) {
            Some(info.clone())
        } else {
            None
        };
        
        // ドロップ先がスタックコンテナかチェック
        let target_stack_container = if let Some(stack) = world.get_component::<StackContainer>(target_id) {
            Some(stack.clone())
        } else {
            None
        };
        
        // ドラッグしてるカードがどのスタックから来たかを調べる
        let source_stack_id = {
            let mut found_stack = None;
            let stacks = world.get_entities_with_component::<StackContainer>();
            
            for &stack_id in &stacks {
                if let Some(stack) = world.get_component::<StackContainer>(stack_id) {
                    if stack.cards.contains(&main_card_id) {
                        found_stack = Some(stack_id);
                        break;
                    }
                }
            }
            
            found_stack
        };
        
        // ドロップが有効かチェック（ソリティアのルールに基づく）
        let mut should_move_cards = false;
        if let (Some(card_info), Some(target_stack)) = (card_info, target_stack_container) {
            // タブローへのドロップのみ許可
            if let crate::ecs::component::StackType::Tableau { .. } = target_stack.stack_type {
                let top_card = target_stack.top_card();
                if let Some(top_id) = top_card {
                    if let Some(top_info) = world.get_component::<crate::ecs::component::CardInfo>(top_id) {
                        // 色が異なり、降順なら配置可能
                        let is_diff_color = card_info.is_red() != top_info.is_red();
                        should_move_cards = is_diff_color && card_info.rank + 1 == top_info.rank;
                    }
                } else {
                    // 空の場札にはKのみ置ける
                    should_move_cards = card_info.rank == 12; // K
                }
            }
        }
        
        // カードを移動（ドロップが有効な場合）
        if should_move_cards {
            // 元のスタックからカードを取り除く
            if let Some(source_id) = source_stack_id {
                if let Some(source_stack) = world.get_component_mut::<StackContainer>(source_id) {
                    // カードの位置を調べる
                    if let Some(card_index) = source_stack.cards.iter().position(|&card| card == main_card_id) {
                        // 該当位置以降のカードをすべて削除
                        let _removed_cards = source_stack.remove_cards_from_index(card_index);
                    }
                }
            }
            
            // 新しいスタックにカードを追加
            if let Some(target_stack) = world.get_component_mut::<StackContainer>(target_id) {
                let start_pos = target_stack.cards.len();
                
                // 各カードを追加
                for &card_id in &dragged_cards {
                    target_stack.add_card(card_id);
                }
                
                // カードの位置を新しいスタックに合わせて更新
                if let Some(target_transform) = world.get_component::<Transform>(target_id) {
                    let base_position = target_transform.position.clone();
                    
                    for (i, &card_id) in dragged_cards.iter().enumerate() {
                        let card_index = start_pos + i;
                        let offset_y = card_index as f64 * crate::constants::STACK_OFFSET_Y;
                        
                        if let Some(transform) = world.get_component_mut::<Transform>(card_id) {
                            transform.position = Vec2::new(
                                base_position.x,
                                base_position.y + offset_y
                            );
                            transform.z_index = card_index as i32;
                        }
                        
                        // ドラッグ状態をリセット
                        if let Some(draggable) = world.get_component_mut::<Draggable>(card_id) {
                            draggable.is_dragging = false;
                        }
                        
                        // 不透明度を元に戻す
                        if let Some(renderable) = world.get_component_mut::<Renderable>(card_id) {
                            renderable.opacity = 1.0;
                        }
                    }
                }
            }
        } else {
            // ドロップが無効なら元の位置に戻す
            self.reset_card_positions(world, &dragged_cards)?;
        }
        
        Ok(())
    }
    
    /// ドラッグしているすべてのカードを取得
    fn get_dragged_cards(&self, world: &World, main_card_id: EntityId) -> Result<Vec<EntityId>, JsValue> {
        let mut dragged_cards = vec![main_card_id];
        
        // カードがどのスタックに属しているか確認
        let stacks = world.get_entities_with_component::<StackContainer>();
        for &stack_id in &stacks {
            if let Some(stack) = world.get_component::<StackContainer>(stack_id) {
                // カードがこのスタックに含まれているか確認
                if let Some(card_index) = stack.cards.iter().position(|&card| card == main_card_id) {
                    // タブローのスタックのみ、カード以降も一緒にドラッグ
                    if let crate::ecs::component::StackType::Tableau { .. } = stack.stack_type {
                        dragged_cards = stack.cards_from_index(card_index);
                    }
                    break;
                }
            }
        }
        
        Ok(dragged_cards)
    }
    
    /// カードの位置を元に戻す
    fn reset_card_positions(&self, world: &mut World, cards: &[EntityId]) -> Result<(), JsValue> {
        for &card_id in cards {
            if let Some(draggable) = world.get_component::<Draggable>(card_id) {
                let original_position = draggable.original_position;
                let original_z_index = draggable.original_z_index;
                
                if let Some(transform) = world.get_component_mut::<Transform>(card_id) {
                    transform.position = original_position;
                    transform.z_index = original_z_index;
                }
                
                // ドラッグ状態をリセット
                if let Some(draggable) = world.get_component_mut::<Draggable>(card_id) {
                    draggable.is_dragging = false;
                }
                
                // 不透明度を元に戻す
                if let Some(renderable) = world.get_component_mut::<Renderable>(card_id) {
                    renderable.opacity = 1.0;
                }
            }
        }
        
        Ok(())
    }
    
    /// マウスクリック位置にあるエンティティを見つける
    fn find_clicked_entity(&self, world: &World, mouse_position: &Vec2) -> Result<Option<EntityId>, JsValue> {
        debug!("🔍 find_clicked_entity: クリック座標=({:.1}, {:.1})", mouse_position.x, mouse_position.y);
        
        let mut clicked_entity = None;
        let mut highest_z_index = -1;
        
        // すべてのエンティティをループして、クリック位置にあるものを探す
        let entities = world.get_all_entities();
        for entity_id in entities {
            // Transformコンポーネントを持つエンティティのみ処理
            if let Some(transform) = world.get_component::<Transform>(entity_id) {
                debug!("📋 エンティティ {} の位置を確認: 位置=({:.1}, {:.1}), サイズ=({:.1}, {:.1}), z_index={}", 
                    entity_id, transform.position.x, transform.position.y, transform.scale.x, transform.scale.y, transform.z_index);
                
                // エンティティの境界を計算
                let min_x = transform.position.x;
                let max_x = transform.position.x + transform.scale.x;
                let min_y = transform.position.y;
                let max_y = transform.position.y + transform.scale.y;
                
                // 点がエンティティの境界内にあるかチェック
                if mouse_position.x >= min_x && mouse_position.x <= max_x && mouse_position.y >= min_y && mouse_position.y <= max_y {
                    debug!("✓ エンティティ {} はクリック座標内にあります", entity_id);
                    
                    // Renderableコンポーネントを持っているか確認
                    if let Some(renderable) = world.get_component::<Renderable>(entity_id) {
                        debug!("✓ エンティティ {} はRenderableを持っています: visible={}, opacity={:.1}", 
                            entity_id, renderable.visible, renderable.opacity);
                        
                        // 表示されているエンティティのみを対象とする
                        if renderable.visible && renderable.opacity > 0.0 {
                            // 最も手前にあるエンティティを選択する（z_indexが大きい方）
                            if transform.z_index > highest_z_index {
                                debug!("⭐ エンティティ {} が現在の最高z_index({})を上回りました: 新z_index={}",
                                    entity_id, highest_z_index, transform.z_index);
                                
                                highest_z_index = transform.z_index;
                                clicked_entity = Some(entity_id);
                            }
                        } else {
                            debug!("✗ エンティティ {} は表示されていないためスキップします", entity_id);
                        }
                    } else {
                        debug!("✗ エンティティ {} はRenderableコンポーネントを持っていないためスキップします", entity_id);
                    }
                } else {
                    debug!("✗ エンティティ {} はクリック座標の範囲外です", entity_id);
                }
            }
        }
        
        if let Some(entity_id) = clicked_entity {
            debug!("🎯 クリックされたエンティティを特定しました: ID={}, z_index={}", entity_id, highest_z_index);
        } else {
            debug!("❌ クリック座標にエンティティは見つかりませんでした");
        }
        
        Ok(clicked_entity)
    }
    
    /// エンティティのクリックを処理
    fn handle_entity_click(&mut self, world: &mut World, entity_id: EntityId) -> Result<(), JsValue> {
        debug!("🖱️ handle_entity_click: エンティティID={}", entity_id);
        
        // エンティティがドラッグ可能か確認
        let is_draggable = world.has_component::<Draggable>(entity_id);
        debug!("🧩 エンティティ {} はドラッグ可能か: {}", entity_id, is_draggable);
        
        if is_draggable {
            // ドラッグ中のエンティティをセット
            self.dragged_entity = Some(entity_id);
            
            // エンティティの元の位置を保存
            if let Some(transform) = world.get_component::<Transform>(entity_id) {
                let original_position = transform.position.clone();
                debug!("📍 エンティティ {} の元の位置を保存: ({:.1}, {:.1})", 
                    entity_id, original_position.x, original_position.y);
            } else {
                debug!("⚠️ エンティティ {} はTransformコンポーネントを持っていません", entity_id);
            }
            
            // オブジェクトの透明度を下げる（ドラッグ中の視覚的フィードバック）
            if let Some(mut renderable) = world.get_component_mut::<Renderable>(entity_id) {
                renderable.opacity = 0.7;
                debug!("🔅 エンティティ {} の透明度を下げました: opacity=0.7", entity_id);
            } else {
                debug!("⚠️ エンティティ {} はRenderableコンポーネントを持っていません", entity_id);
            }
            
            // z-indexを一時的に上げて、他のオブジェクトの上に表示
            if let Some(mut transform) = world.get_component_mut::<Transform>(entity_id) {
                self.original_z_index = transform.z_index;
                transform.z_index = 1000; // 一時的に最前面に
                debug!("📊 エンティティ {} のz_indexを一時的に上げました: {} -> 1000", 
                    entity_id, self.original_z_index);
            }
        } else {
            debug!("❌ エンティティ {} はドラッグ可能ではありません", entity_id);
        }
        
        Ok(())
    }
}

impl System for DragSystem {
    fn name(&self) -> &'static str {
        "DragSystem"
    }
    
    fn phase(&self) -> SystemPhase {
        SystemPhase::Input  // 入力フェーズで実行
    }
    
    fn priority(&self) -> SystemPriority {
        SystemPriority::new(1)  // InputSystemの後に実行
    }
    
    fn run(
        &mut self,
        world: &mut World,
        resources: &mut ResourceManager,
        _delta_time: f32,
    ) -> Result<(), JsValue> {
        // 結果を取得
        let result = self.update(world, resources);
        
        // フレームの最後にクリックフラグをリセット
        if let Some(input_state) = resources.get_mut::<InputState>() {
            if input_state.is_mouse_clicked {
                debug!("🖱️ クリックフラグをリセットしました");
                input_state.is_mouse_clicked = false;
            }
        }
        
        result
    }
}

impl DragSystem {
    pub fn update(&mut self, world: &mut World, resources: &ResourceManager) -> Result<(), JsValue> {
        // マウスの状態を取得
        let mouse_state = match resources.get::<InputState>() {
            Some(state) => state,
            None => return Ok(()),
        };
        
        debug!("🖱️ マウスの状態: 位置=({:.1}, {:.1}), 左ボタン={}, 右ボタン={}, 前回の左ボタン={}, クリック={}", 
            mouse_state.mouse_position.x, mouse_state.mouse_position.y, 
            mouse_state.mouse_buttons[0], mouse_state.mouse_buttons[2], 
            self.left_button_pressed_prev, mouse_state.is_mouse_clicked);
        
        // 前のフレームからのマウス位置の変化を計算
        let mouse_delta = Vec2::new(
            mouse_state.mouse_position.x - self.last_mouse_pos.x,
            mouse_state.mouse_position.y - self.last_mouse_pos.y,
        );
        debug!("🔄 マウス移動量: ({:.1}, {:.1})", mouse_delta.x, mouse_delta.y);
        
        // マウスの位置を更新
        self.last_mouse_pos = mouse_state.mouse_position.clone();
        
        // マウスがクリックされたとき（マウスボタン状態の変化または明示的なクリックフラグ）
        if (mouse_state.mouse_buttons[0] && !self.left_button_pressed_prev) || mouse_state.is_mouse_clicked {
            debug!("👇 マウスクリックを検出: ボタン状態={}, 前回状態={}, クリックフラグ={}",
                  mouse_state.mouse_buttons[0], self.left_button_pressed_prev, mouse_state.is_mouse_clicked);
            
            // クリックされたエンティティを検索
            if let Some(entity_id) = self.find_clicked_entity(world, &mouse_state.mouse_position)? {
                debug!("🎯 クリックされたエンティティを検出: {}", entity_id);
                self.handle_entity_click(world, entity_id)?;
            } else {
                debug!("🚫 クリック位置にエンティティが見つかりませんでした");
            }
        }
        
        // マウスの左ボタンが離されたとき
        if !mouse_state.mouse_buttons[0] && self.left_button_pressed_prev {
            debug!("👆 マウス左ボタンが離されました");
            
            // ドラッグ中のエンティティがあれば終了処理を行う
            if let Some(dragged_entity) = self.dragged_entity {
                debug!("🔚 ドラッグ終了: エンティティID={}", dragged_entity);
                self.end_drag(world)?;
                self.dragged_entity = None; // ドラッグ状態をリセット
            } else {
                debug!("ℹ️ ドラッグ中のエンティティはありませんでした");
            }
        }
        
        // ドラッグ中の処理
        if mouse_state.mouse_buttons[0] && self.dragged_entity.is_some() {
            let entity_id = self.dragged_entity.unwrap();
            debug!("🔄 ドラッグ中: エンティティID={}", entity_id);
            
            if let Some(mut transform) = world.get_component_mut::<Transform>(entity_id) {
                // マウスの移動に合わせてオブジェクトを移動
                transform.position.x += mouse_delta.x;
                transform.position.y += mouse_delta.y;
                debug!("📍 エンティティ {} の位置を更新: ({:.1}, {:.1})", 
                    entity_id, transform.position.x, transform.position.y);
                
                // positionをコピーしてから、highlight_drop_targetを呼び出す
                let position_copy = transform.position.clone();
                
                // ドロップ先の候補をハイライト
                self.highlight_drop_target(world, &position_copy)?;
            } else {
                debug!("⚠️ ドラッグ中のエンティティ {} はTransformコンポーネントを持っていません", entity_id);
            }
        }
        
        // 前フレームのマウス状態を更新
        self.left_button_pressed_prev = mouse_state.mouse_buttons[0];
        
        Ok(())
    }
}
