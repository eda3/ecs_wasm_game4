use wasm_bindgen::prelude::*;
use crate::ecs::world::World;
use crate::ecs::system::{System, SystemPhase, SystemPriority};
use crate::ecs::resources::{ResourceManager, InputState};
use crate::ecs::component::{Transform, Draggable, Clickable, StackContainer, StackType, Droppable};
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
}

impl DragSystem {
    /// 新しいドラッグシステムを作成
    pub fn new() -> Self {
        Self {
            dragged_entity: None,
            drag_start_position: Vec2::zero(),
            drag_started: false,
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
        // 必要な情報を先に取得
        let transform_position;
        let transform_z_index;
        
        // 1. エンティティの現在位置を先に取得
        {
            if let Some(transform) = world.get_component::<Transform>(entity_id) {
                transform_position = transform.position;
                transform_z_index = transform.z_index;
            } else {
                // Transformがなければ処理を中止
                return Ok(());
            }
        }
        
        // 2. ドラッグオフセットを計算
        let drag_offset = Vec2::new(
            mouse_position.x - transform_position.x,
            mouse_position.y - transform_position.y,
        );
        
        // 3. ドラッグ可能コンポーネントを更新
        if let Some(draggable) = world.get_component_mut::<Draggable>(entity_id) {
            draggable.is_dragging = true;
            draggable.original_position = transform_position;
            draggable.original_z_index = transform_z_index;
            draggable.drag_offset = drag_offset;
        }
        
        // 4. レンダラブルコンポーネントの不透明度を下げる
        if let Some(renderable) = world.get_component_mut::<crate::ecs::component::Renderable>(entity_id) {
            renderable.opacity = DRAG_OPACITY;
        }
        
        // 5. カードがタブローのスタックにある場合、そのカード以降のカードも一緒にドラッグ
        let mut cards_to_drag = Vec::new();
        
        // カードがどのスタックに属しているか確認
        let stacks = world.get_entities_with_component::<crate::ecs::component::StackContainer>();
        for &stack_id in &stacks {
            if let Some(stack) = world.get_component::<crate::ecs::component::StackContainer>(stack_id) {
                // カードがこのスタックに含まれているか確認
                if let Some(card_index) = stack.cards.iter().position(|&card| card == entity_id) {
                    // タブローのスタックのみ、カード以降も一緒にドラッグ
                    if let crate::ecs::component::StackType::Tableau { .. } = stack.stack_type {
                        // 選択したカード以降のカードを追加
                        cards_to_drag = stack.cards_from_index(card_index);
                        
                        // カードがタブロー内にあり、複数カードをドラッグする場合
                        if cards_to_drag.len() > 1 {
                            // 一番上のカード以外の不透明度も下げる
                            for (i, &card_id) in cards_to_drag.iter().enumerate().skip(1) {
                                if let Some(card_renderable) = world.get_component_mut::<crate::ecs::component::Renderable>(card_id) {
                                    card_renderable.opacity = DRAG_OPACITY;
                                }
                                
                                // カードの位置を調整（重ねて表示）
                                // 1. 必要なデータを先に取得
                                let position;
                                let z_index;
                                {
                                    if let Some(card_transform) = world.get_component::<crate::ecs::component::Transform>(card_id) {
                                        position = card_transform.position.clone();
                                        z_index = card_transform.z_index;
                                    } else {
                                        continue;
                                    }
                                }
                                
                                // 2. Draggableコンポーネントを更新
                                if let Some(card_draggable) = world.get_component_mut::<Draggable>(card_id) {
                                    card_draggable.original_position = position;
                                    card_draggable.original_z_index = z_index;
                                    // 実際にドラッグされてるようにフラグを設定
                                    card_draggable.is_dragging = true;
                                }
                                
                                // 3. 別のスコープでTransformコンポーネントを再度取得して更新
                                if let Some(card_transform) = world.get_component_mut::<crate::ecs::component::Transform>(card_id) {
                                    // Z-indexを調整して重なる順序を維持
                                    card_transform.z_index = 1000 + i as i32;
                                }
                            }
                        }
                    }
                    break;
                }
            }
        }
        
        // 6. ドラッグ中のエンティティを記録
        self.dragged_entity = Some(entity_id);
        self.drag_start_position = mouse_position;
        self.drag_started = true;
        
        debug!("🖱️ エンティティ {} のドラッグを開始（一緒にドラッグするカード: {}枚）", entity_id, cards_to_drag.len());
        
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
        let stacks = world.get_entities_with_component::<crate::ecs::component::StackContainer>();
        for &stack_id in &stacks {
            if let Some(stack) = world.get_component::<crate::ecs::component::StackContainer>(stack_id) {
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
    fn end_drag(&mut self, world: &mut World) -> Result<(), JsValue> {
        if let Some(entity_id) = self.dragged_entity {
            // 現在の位置とドロップターゲットの情報を先に取得
            let current_position;
            let drop_target;
            let valid_drop;
            
            {
                // トランスフォームの現在位置を取得
                if let Some(transform) = world.get_component::<Transform>(entity_id) {
                    current_position = transform.position;
                } else {
                    current_position = Vec2::zero();
                }
                
                // ドロップターゲットを見つける
                drop_target = self.find_drop_target(world, current_position, entity_id as usize)?;
                
                // ドロップが有効かチェック
                valid_drop = if let Some(target_id) = drop_target {
                    self.is_valid_drop(world, entity_id as usize, target_id)?
                } else {
                    false
                };
            }
            
            // ドラッグしているカードと一緒にドラッグしている他のカードを取得
            let dragged_cards = self.get_dragged_cards(world, entity_id)?;
            
            // ドロップターゲットが有効なら
            if let Some(target_id) = drop_target {
                if valid_drop {
                    // ドラッグを処理する
                    if dragged_cards.len() > 1 {
                        // 複数カードのドロップを処理
                        self.process_multi_card_drop(world, dragged_cards, target_id)?;
                    } else {
                        // 単一カードのドロップを処理
                        self.process_drop(world, entity_id, target_id)?;
                    }
                } else {
                    // 無効なドロップの場合は元の位置に戻す
                    self.reset_card_positions(world, &dragged_cards)?;
                }
            } else {
                // ドロップターゲットがない場合は元の位置に戻す
                self.reset_card_positions(world, &dragged_cards)?;
            }
            
            // ドラッグ状態をリセット
            self.dragged_entity = None;
        }
        
        Ok(())
    }
    
    /// ドロップターゲットを見つける
    fn find_drop_target(&self, world: &World, position: Vec2, dragged_entity: usize) -> Result<Option<usize>, JsValue> {
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
    fn is_valid_drop(&self, world: &World, dragged_entity: usize, target_entity: usize) -> Result<bool, JsValue> {
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
        let target_stack_container = if let Some(stack) = world.get_component::<crate::ecs::component::StackContainer>(drop_target) {
            Some(stack.clone())
        } else {
            None
        };
        
        // ドラッグしてるカードがどのスタックから来たかを調べる
        let source_stack_id = {
            let mut found_stack = None;
            let stacks = world.get_entities_with_component::<crate::ecs::component::StackContainer>();
            
            for &stack_id in &stacks {
                if let Some(stack) = world.get_component::<crate::ecs::component::StackContainer>(stack_id) {
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
                if let Some(source_stack) = world.get_component_mut::<crate::ecs::component::StackContainer>(source_id) {
                    source_stack.remove_card(dragged_entity);
                }
            }
            
            // 新しいスタックにカードを追加
            if let Some(target_stack) = world.get_component_mut::<crate::ecs::component::StackContainer>(drop_target) {
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
                    if let Some(target_transform) = world.get_component::<crate::ecs::component::Transform>(drop_target) {
                        drop_position = target_transform.position.clone();
                    } else {
                        drop_position = Vec2::zero();
                    }
                }
                
                // 3. スタックのカード数に基づいて位置を計算
                let offset_y = cards_count as f64 * crate::constants::STACK_OFFSET_Y;
                
                // 4. ドラッグしたカードのTransformを更新
                if let Some(transform) = world.get_component_mut::<crate::ecs::component::Transform>(dragged_entity) {
                    transform.position = Vec2::new(
                        drop_position.x,
                        drop_position.y + offset_y
                    );
                    transform.z_index = cards_count as i32;
                }
            }
        } else {
            // ドロップが無効なら元の位置に戻す
            if let Some(draggable) = world.get_component::<crate::ecs::component::Draggable>(dragged_entity) {
                let original_position = draggable.original_position;
                let original_z_index = draggable.original_z_index;
                
                if let Some(transform) = world.get_component_mut::<crate::ecs::component::Transform>(dragged_entity) {
                    transform.position = original_position;
                    transform.z_index = original_z_index;
                }
            }
        }
        
        // ドラッグ状態をリセット
        if let Some(draggable) = world.get_component_mut::<crate::ecs::component::Draggable>(dragged_entity) {
            draggable.is_dragging = false;
        }
        
        // レンダラブルコンポーネントの不透明度を元に戻す
        if let Some(renderable) = world.get_component_mut::<crate::ecs::component::Renderable>(dragged_entity) {
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
        let target_stack_container = if let Some(stack) = world.get_component::<crate::ecs::component::StackContainer>(target_id) {
            Some(stack.clone())
        } else {
            None
        };
        
        // ドラッグしてるカードがどのスタックから来たかを調べる
        let source_stack_id = {
            let mut found_stack = None;
            let stacks = world.get_entities_with_component::<crate::ecs::component::StackContainer>();
            
            for &stack_id in &stacks {
                if let Some(stack) = world.get_component::<crate::ecs::component::StackContainer>(stack_id) {
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
                if let Some(source_stack) = world.get_component_mut::<crate::ecs::component::StackContainer>(source_id) {
                    // カードの位置を調べる
                    if let Some(card_index) = source_stack.cards.iter().position(|&card| card == main_card_id) {
                        // 該当位置以降のカードをすべて削除
                        let _removed_cards = source_stack.remove_cards_from_index(card_index);
                    }
                }
            }
            
            // 新しいスタックにカードを追加
            if let Some(target_stack) = world.get_component_mut::<crate::ecs::component::StackContainer>(target_id) {
                let start_pos = target_stack.cards.len();
                
                // 各カードを追加
                for &card_id in &dragged_cards {
                    target_stack.add_card(card_id);
                }
                
                // カードの位置を新しいスタックに合わせて更新
                if let Some(target_transform) = world.get_component::<crate::ecs::component::Transform>(target_id) {
                    let base_position = target_transform.position.clone();
                    
                    for (i, &card_id) in dragged_cards.iter().enumerate() {
                        let card_index = start_pos + i;
                        let offset_y = card_index as f64 * crate::constants::STACK_OFFSET_Y;
                        
                        if let Some(transform) = world.get_component_mut::<crate::ecs::component::Transform>(card_id) {
                            transform.position = Vec2::new(
                                base_position.x,
                                base_position.y + offset_y
                            );
                            transform.z_index = card_index as i32;
                        }
                        
                        // ドラッグ状態をリセット
                        if let Some(draggable) = world.get_component_mut::<crate::ecs::component::Draggable>(card_id) {
                            draggable.is_dragging = false;
                        }
                        
                        // 不透明度を元に戻す
                        if let Some(renderable) = world.get_component_mut::<crate::ecs::component::Renderable>(card_id) {
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
        let stacks = world.get_entities_with_component::<crate::ecs::component::StackContainer>();
        for &stack_id in &stacks {
            if let Some(stack) = world.get_component::<crate::ecs::component::StackContainer>(stack_id) {
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
            if let Some(draggable) = world.get_component::<crate::ecs::component::Draggable>(card_id) {
                let original_position = draggable.original_position;
                let original_z_index = draggable.original_z_index;
                
                if let Some(transform) = world.get_component_mut::<crate::ecs::component::Transform>(card_id) {
                    transform.position = original_position;
                    transform.z_index = original_z_index;
                }
                
                // ドラッグ状態をリセット
                if let Some(draggable) = world.get_component_mut::<crate::ecs::component::Draggable>(card_id) {
                    draggable.is_dragging = false;
                }
                
                // 不透明度を元に戻す
                if let Some(renderable) = world.get_component_mut::<crate::ecs::component::Renderable>(card_id) {
                    renderable.opacity = 1.0;
                }
            }
        }
        
        Ok(())
    }
    
    /// クリックされたエンティティを探す
    fn find_clicked_entity(&self, world: &World, position: Vec2) -> Result<Option<EntityId>, JsValue> {
        // ドラッグ可能なエンティティを探す
        let draggable_entities = world.get_entities_with_component::<Draggable>();
        
        let mut potential_target = None;
        let mut highest_z_index = -1;
        
        // すべてのドラッグ可能なエンティティをチェック
        for &entity_id in &draggable_entities {
            if let Some(transform) = world.get_component::<Transform>(entity_id) {
                if let Some(draggable) = world.get_component::<Draggable>(entity_id) {
                    // エンティティの領域内にマウスがあるかチェック
                    if position.x >= transform.position.x
                        && position.x <= transform.position.x + draggable.width
                        && position.y >= transform.position.y
                        && position.y <= transform.position.y + draggable.height
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
    
    /// エンティティのクリックを処理
    fn handle_entity_click(&mut self, world: &mut World, entity_id: EntityId, mouse_position: Vec2) -> Result<(), JsValue> {
        // エンティティがドラッグ可能かチェック
        if world.has_component::<Draggable>(entity_id) {
            // ドラッグを開始
            self.start_drag(world, entity_id, mouse_position)?;
            debug!("🖱️ エンティティ {} のドラッグを開始", entity_id);
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
        self.update(world, resources)
    }
}

impl DragSystem {
    fn update(&mut self, world: &mut World, resource_manager: &ResourceManager) -> Result<(), JsValue> {
        let input_state = resource_manager.get::<InputState>();
        
        // input_stateがNoneの場合は早期リターン
        if input_state.is_none() {
            return Ok(());
        }
        
        let input_state = input_state.unwrap();
        
        // マウスイベントを処理
        // クリックされたエンティティを見つける
        let clicked_entity = if input_state.is_mouse_clicked {
            self.find_clicked_entity(world, input_state.mouse_position)?
        } else {
            None
        };
        
        // クリックされたエンティティがあれば処理
        if let Some(entity_id) = clicked_entity {
            self.handle_entity_click(world, entity_id, input_state.mouse_position)?;
        }
        
        // ドラッグ処理
        if let Some(entity_id) = self.dragged_entity {
            if input_state.is_mouse_down {
                // ドラッグ中の更新
                self.update_drag(world, entity_id, input_state.mouse_position)?;
            }
            // マウスボタンが離された瞬間
            else if !input_state.is_mouse_down {
                self.end_drag(world)?;
            }
        }
        
        Ok(())
    }
} 