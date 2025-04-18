use wasm_bindgen::prelude::*;
use crate::ecs::world::World;
use crate::ecs::entity::EntityId;
use crate::ecs::component::{Transform, CardInfo, Renderable, Draggable, Clickable, ClickHandlerType};
use crate::constants::{CARD_WIDTH, CARD_HEIGHT, CARD_BORDER_RADIUS};

/// カードを作成する関数
/// プレイヤーが操作するトランプカードのエンティティを作成
pub fn create_card(
    world: &mut World,
    suit: u8,
    rank: u8,
    x: f64,
    y: f64,
    face_up: bool,
    z_index: i32,
) -> Result<EntityId, JsValue> {
    // 新しいエンティティを作成
    let entity_id = world.create_entity()?;
    
    // トランスフォームコンポーネントを追加
    let transform = Transform::new(x, y).with_z_index(z_index);
    world.add_component(entity_id, transform)?;
    
    // カード情報コンポーネントを追加
    let mut card_info = CardInfo::new(suit, rank);
    card_info.face_up = face_up;
    world.add_component(entity_id, card_info)?;
    
    // レンダラブルコンポーネントを追加
    let renderable = Renderable::card(CARD_WIDTH, CARD_HEIGHT);
    world.add_component(entity_id, renderable)?;
    
    // ドラッグ可能コンポーネントを追加（面が表の場合のみ）
    if face_up {
        let draggable = Draggable::new();
        world.add_component(entity_id, draggable)?;
    }
    
    // クリック可能コンポーネントを追加
    let clickable = Clickable::new(ClickHandlerType::FlipCard);
    world.add_component(entity_id, clickable)?;
    
    Ok(entity_id)
}

/// カードの表面と裏面を切り替える関数
pub fn flip_card(world: &mut World, card_id: EntityId) -> Result<(), JsValue> {
    // カード情報コンポーネントを取得
    if let Some(card_info) = world.get_component_mut::<CardInfo>(card_id) {
        // 表裏を反転
        card_info.face_up = !card_info.face_up;
        
        // 表向きになった場合、ドラッグ可能にする
        if card_info.face_up {
            if !world.has_component::<Draggable>(card_id) {
                world.add_component(card_id, Draggable::new())?;
            }
        } else {
            // 裏向きになった場合、ドラッグ不可にする（必要に応じて）
            // world.remove_component::<Draggable>(card_id);
        }
        
        Ok(())
    } else {
        Err(JsValue::from_str(&format!(
            "エンティティID: {} にCardInfoコンポーネントが見つかりません",
            card_id
        )))
    }
}

/// カードがもう一方のカードの上に積み重ね可能かチェックする関数
pub fn can_stack_card(world: &World, source_card_id: EntityId, target_card_id: EntityId) -> bool {
    // 両方のカード情報を取得
    let source_card = match world.get_component::<CardInfo>(source_card_id) {
        Some(card) => card,
        None => return false,
    };
    
    let target_card = match world.get_component::<CardInfo>(target_card_id) {
        Some(card) => card,
        None => return false,
    };
    
    // ソリティアのルールに基づいて積み重ね可能かチェック
    // タブロー（場札）のルール: 色が異なり、ランクが1つ大きいカードの上に置ける
    let different_colors = source_card.color != target_card.color;
    let descending_rank = source_card.rank + 1 == target_card.rank;
    
    different_colors && descending_rank
}

/// ファウンデーション（組み札）にカードを積み重ね可能かチェックする関数
pub fn can_stack_on_foundation(world: &World, card_id: EntityId, foundation_suit: u8) -> bool {
    // カード情報を取得
    let card = match world.get_component::<CardInfo>(card_id) {
        Some(card) => card,
        None => return false,
    };
    
    // カードのスートが一致するか
    if card.suit != foundation_suit {
        return false;
    }
    
    // ファウンデーションのルール: 同じスートのカードをA(0)から順番に積み上げる
    // ファウンデーションが空の場合は、そのスートのA(0)のみ置ける
    card.rank == 0
}

/// 既存のファウンデーション（組み札）にカードを積み重ね可能かチェックする関数
pub fn can_stack_on_existing_foundation(
    world: &World,
    card_id: EntityId,
    top_foundation_card_id: EntityId,
) -> bool {
    // 両方のカード情報を取得
    let card = match world.get_component::<CardInfo>(card_id) {
        Some(card) => card,
        None => return false,
    };
    
    let top_card = match world.get_component::<CardInfo>(top_foundation_card_id) {
        Some(card) => card,
        None => return false,
    };
    
    // スートが一致し、ランクが1つ上のカードのみ置ける
    card.suit == top_card.suit && card.rank == top_card.rank + 1
}

/// 完全なカードデッキ（52枚）を作成
pub fn create_deck(world: &mut World, x: f64, y: f64) -> Result<Vec<EntityId>, JsValue> {
    let mut deck = Vec::with_capacity(52);
    
    // 各スート（0-3）、各ランク（0-12）のカードを作成
    for suit in 0..4 {
        for rank in 0..13 {
            // Z-indexを順番に増やして重なりを制御
            let z_index = (suit * 13 + rank) as i32;
            
            // カードを作成（初期状態は裏向き）
            let card_id = create_card(world, suit, rank, x, y, false, z_index)?;
            deck.push(card_id);
        }
    }
    
    Ok(deck)
}

/// カードデッキをシャッフル
pub fn shuffle_deck(deck: &mut Vec<EntityId>) {
    use rand::seq::SliceRandom;
    let mut rng = rand::thread_rng();
    deck.shuffle(&mut rng);
}

/// カードの位置を設定
pub fn set_card_position(world: &mut World, card_id: EntityId, x: f64, y: f64, z_index: i32) -> Result<(), JsValue> {
    if let Some(transform) = world.get_component_mut::<Transform>(card_id) {
        transform.position.x = x;
        transform.position.y = y;
        transform.z_index = z_index;
        Ok(())
    } else {
        Err(JsValue::from_str(&format!(
            "エンティティID: {} にTransformコンポーネントが見つかりません",
            card_id
        )))
    }
}

/// カードがドラッグ可能かどうかを設定
pub fn set_card_draggable(world: &mut World, card_id: EntityId, draggable: bool) -> Result<(), JsValue> {
    if draggable {
        if !world.has_component::<Draggable>(card_id) {
            world.add_component(card_id, Draggable::new())?;
        }
    } else {
        world.remove_component::<Draggable>(card_id);
    }
    
    Ok(())
} 