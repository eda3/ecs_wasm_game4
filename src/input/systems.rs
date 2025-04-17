use wasm_bindgen::prelude::*;
use crate::ecs::world::World;
use crate::ecs::system::{System, SystemPhase, SystemPriority};
use crate::ecs::resources::{ResourceManager, InputState};
use crate::ecs::component::{Transform, Draggable, Clickable, StackContainer, StackType, Droppable};
use crate::ecs::entity::EntityId;
use crate::input::input_handler::InputHandler;
use crate::utils::Vec2;
use crate::constants::{DRAG_THRESHOLD, DRAG_OPACITY};
use log::{debug, error};

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
        resources: &mut ResourceManager,
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
            Some(state) => state.clone(),  // クローンして所有権の問題を回避
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
                self.process_click(world, _resources, entity_id)?;
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
        // エンティティのドラッグ可能コンポーネントを更新
        if let Some(draggable) = world.get_component_mut::<Draggable>(entity_id) {
            draggable.is_dragging = true;
            
            // エンティティの現在位置を取得
            if let Some(transform) = world.get_component::<Transform>(entity_id) {
                draggable.original_position = transform.position;
                
                // ドラッグオフセットを計算（クリック位置とエンティティの左上の差）
                draggable.drag_offset = Vec2::new(
                    mouse_position.x - transform.position.x,
                    mouse_position.y - transform.position.y,
                );
            }
        }
        
        // レンダラブルコンポーネントの不透明度を下げる
        if let Some(renderable) = world.get_component_mut::<crate::ecs::component::Renderable>(entity_id) {
            renderable.opacity = DRAG_OPACITY;
        }
        
        // ドラッグ中のエンティティを記録
        self.dragged_entity = Some(entity_id);
        self.drag_start_position = mouse_position;
        self.drag_started = true;
        
        debug!("🖱️ エンティティ {} のドラッグを開始", entity_id);
        
        Ok(())
    }
    
    /// ドラッグ中の更新
    fn update_drag(&mut self, world: &mut World, mouse_position: Vec2) -> Result<(), JsValue> {
        if let Some(entity_id) = self.dragged_entity {
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
            
            // ドラッグ中の子要素も一緒に移動
            let drag_children = if let Some(draggable) = world.get_component::<Draggable>(entity_id) {
                draggable.drag_children
            } else {
                false
            };
            
            if drag_children {
                // スタックコンテナを持つ場合、カードを一緒に移動
                if let Some(_stack) = world.get_component::<StackContainer>(entity_id) {
                    // スタック内のカードも移動
                    // 実際の実装はもっと複雑になるが、ここではシンプルに
                }
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
            
            // ドロップターゲットが有効なら
            if let Some(target_id) = drop_target {
                if valid_drop {
                    // ドラッグを処理する
                    self.process_drop(world, entity_id, target_id)?;
                } else {
                    // 無効なドロップの場合は元の位置に戻す
                    if let Some(draggable) = world.get_component_mut::<Draggable>(entity_id) {
                        if let Some(transform) = world.get_component_mut::<Transform>(entity_id) {
                            transform.position = draggable.original_position;
                            transform.z_index = draggable.original_z_index;
                        }
                    }
                }
            } else {
                // ドロップターゲットがない場合は元の位置に戻す
                if let Some(draggable) = world.get_component_mut::<Draggable>(entity_id) {
                    if let Some(transform) = world.get_component_mut::<Transform>(entity_id) {
                        transform.position = draggable.original_position;
                        transform.z_index = draggable.original_z_index;
                    }
                }
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
        
        // ドロップ対象エンティティの情報を先に取得
        let drop_position;
        let original_position;
        let original_z_index;
        
        {
            // ドロップ先の位置を取得
            if let Some(target_transform) = world.get_component::<Transform>(drop_target) {
                drop_position = target_transform.position.clone();
            } else {
                drop_position = Vec2::zero();
            }
            
            // ドラッグしたエンティティの元の位置を取得
            if let Some(draggable) = world.get_component::<Draggable>(dragged_entity) {
                original_position = draggable.original_position;
                original_z_index = draggable.original_z_index;
            } else {
                original_position = Vec2::zero();
                original_z_index = 0;
            }
        }
        
        // ドラッグしたエンティティの状態を更新
        if let Some(draggable) = world.get_component_mut::<Draggable>(dragged_entity) {
            draggable.is_dragging = false;
            
            // ドロップ先に応じた処理
            // ここで具体的なゲームロジックを実装
            // 例: カードをデッキに追加、アイテムをインベントリに配置など
            
            // 現在は単純に位置を更新するだけの例
            if let Some(transform) = world.get_component_mut::<Transform>(dragged_entity) {
                // ドロップ先の上に配置（例としてオフセットを追加）
                transform.position = Vec2::new(
                    drop_position.x + 10.0,
                    drop_position.y + 10.0
                );
                transform.z_index = original_z_index;
            }
        }
        
        // ドロップイベントを発火させる
        // ここでゲーム内のイベントシステムを使ってドロップイベントを通知できる
        
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
        let input_state = resource_manager.get_resource::<InputState>();
        
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
                self.update_drag(world, input_state.mouse_position)?;
            }
            // マウスボタンが離された瞬間
            else if !input_state.is_mouse_down {
                self.end_drag(world)?;
            }
        }
        
        Ok(())
    }
} 