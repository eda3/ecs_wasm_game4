use wasm_bindgen::prelude::*;
use crate::ecs::world::World;
use crate::ecs::entity::EntityId;
use crate::ecs::component::{Transform, CardInfo, StackContainer, StackType, Clickable, ClickHandlerType};
use crate::game::card;
use crate::constants::{
    STOCK_X, STOCK_Y, WASTE_X, WASTE_Y,
    FOUNDATION_START_X, FOUNDATION_START_Y,
    TABLEAU_START_X, TABLEAU_START_Y,
    CARD_SPACING_X, STACK_OFFSET_Y,
};

/// ソリティア（クロンダイク）ゲームのボードをセットアップ
pub fn setup_solitaire_board(world: &mut World) -> Result<(), JsValue> {
    // デッキを作成
    let mut deck = card::create_deck(world, STOCK_X, STOCK_Y)?;
    
    // デッキをシャッフル
    card::shuffle_deck(&mut deck);
    
    // ストック（山札）を作成
    let stock_id = create_stock(world, deck.clone())?;
    
    // ウェイスト（捨て札）を作成
    let waste_id = create_waste(world)?;
    
    // タブロー（場札）を作成 - 7列
    let tableau_ids = create_tableau(world)?;
    
    // タブローにカードを配る
    deal_cards_to_tableau(world, &mut deck, &tableau_ids)?;
    
    // ファウンデーション（組み札）を作成 - 4スート
    let foundation_ids = create_foundations(world)?;
    
    // 残りのカードをストックに追加
    add_cards_to_stock(world, stock_id, &deck)?;
    
    Ok(())
}

/// ストック（山札）を作成
fn create_stock(world: &mut World, cards: Vec<EntityId>) -> Result<EntityId, JsValue> {
    // ストックのエンティティを作成
    let stock_id = world.create_entity()?;
    
    // トランスフォームコンポーネントを追加
    let transform = Transform::new(STOCK_X, STOCK_Y);
    world.add_component(stock_id, transform)?;
    
    // スタックコンテナコンポーネントを追加
    let stack = StackContainer::new(StackType::Stock);
    world.add_component(stock_id, stack)?;
    
    // クリック可能コンポーネントを追加
    let clickable = Clickable::new(ClickHandlerType::DrawFromStock);
    world.add_component(stock_id, clickable)?;
    
    Ok(stock_id)
}

/// ウェイスト（捨て札）を作成
fn create_waste(world: &mut World) -> Result<EntityId, JsValue> {
    // ウェイストのエンティティを作成
    let waste_id = world.create_entity()?;
    
    // トランスフォームコンポーネントを追加
    let transform = Transform::new(WASTE_X, WASTE_Y);
    world.add_component(waste_id, transform)?;
    
    // スタックコンテナコンポーネントを追加
    let stack = StackContainer::new(StackType::Waste);
    world.add_component(waste_id, stack)?;
    
    // クリック可能コンポーネントを追加
    let clickable = Clickable::new(ClickHandlerType::DrawFromWaste);
    world.add_component(waste_id, clickable)?;
    
    Ok(waste_id)
}

/// タブロー（場札）を作成 - 7列
fn create_tableau(world: &mut World) -> Result<Vec<EntityId>, JsValue> {
    let mut tableau_ids = Vec::with_capacity(7);
    
    for i in 0..7 {
        // 各列のエンティティを作成
        let tableau_id = world.create_entity()?;
        
        // 位置を計算（横に並べる）
        let x = TABLEAU_START_X + (i as f64 * CARD_SPACING_X * 1.5);
        let y = TABLEAU_START_Y;
        
        // トランスフォームコンポーネントを追加
        let transform = Transform::new(x, y);
        world.add_component(tableau_id, transform)?;
        
        // スタックコンテナコンポーネントを追加
        let stack = StackContainer::new(StackType::Tableau { column: i });
        world.add_component(tableau_id, stack)?;
        
        // クリック可能コンポーネントを追加
        let clickable = Clickable::new(ClickHandlerType::DrawFromTableau { column: i });
        world.add_component(tableau_id, clickable)?;
        
        tableau_ids.push(tableau_id);
    }
    
    Ok(tableau_ids)
}

/// ファウンデーション（組み札）を作成 - 4スート
fn create_foundations(world: &mut World) -> Result<Vec<EntityId>, JsValue> {
    let mut foundation_ids = Vec::with_capacity(4);
    
    for i in 0..4 {
        // 各スートのエンティティを作成
        let foundation_id = world.create_entity()?;
        
        // 位置を計算（横に並べる）
        let x = FOUNDATION_START_X + (i as f64 * CARD_SPACING_X * 1.5);
        let y = FOUNDATION_START_Y;
        
        // トランスフォームコンポーネントを追加
        let transform = Transform::new(x, y);
        world.add_component(foundation_id, transform)?;
        
        // スタックコンテナコンポーネントを追加
        let stack = StackContainer::new(StackType::Foundation { suit: i });
        world.add_component(foundation_id, stack)?;
        
        // クリック可能コンポーネントを追加
        let clickable = Clickable::new(ClickHandlerType::DrawFromFoundation { stack: i });
        world.add_component(foundation_id, clickable)?;
        
        foundation_ids.push(foundation_id);
    }
    
    Ok(foundation_ids)
}

/// タブローにカードを配る
fn deal_cards_to_tableau(
    world: &mut World,
    deck: &mut Vec<EntityId>,
    tableau_ids: &[EntityId],
) -> Result<(), JsValue> {
    // ソリティアのルールに従って、タブローの各列にカードを配る
    // 1列目に1枚、2列目に2枚、...、7列目に7枚
    for (i, &tableau_id) in tableau_ids.iter().enumerate() {
        let num_cards = i + 1;
        
        // 先にトランスフォーム情報を取得して、必要な値をコピーする
        let base_x;
        let base_y;
        
        if let Some(transform) = world.get_component::<Transform>(tableau_id) {
            base_x = transform.position.x;
            base_y = transform.position.y;
        } else {
            return Err(JsValue::from_str("タブローのトランスフォームが見つかりません"));
        }
        
        // 各列に必要な枚数のカードを配る
        let mut tableau_cards = Vec::with_capacity(num_cards);
        
        for j in 0..num_cards {
            if deck.is_empty() {
                break;
            }
            
            // デッキから1枚取り出す
            let card_id = deck.pop().unwrap();
            
            // カードの位置を設定
            let y_offset = j as f64 * STACK_OFFSET_Y;
            card::set_card_position(world, card_id, base_x, base_y + y_offset, j as i32)?;
            
            // 最後のカードだけ表向きにする
            if j == num_cards - 1 {
                card::flip_card(world, card_id)?;
            }
            
            // 後でスタックに追加するために一時的に保存
            tableau_cards.push(card_id);
        }
        
        // 最後にカードをタブローのスタックに追加
        if let Some(tableau) = world.get_component_mut::<StackContainer>(tableau_id) {
            for card_id in tableau_cards {
                tableau.add_card(card_id);
            }
        }
    }
    
    Ok(())
}

/// 残りのカードをストックに追加
fn add_cards_to_stock(
    world: &mut World,
    stock_id: EntityId,
    cards: &[EntityId],
) -> Result<(), JsValue> {
    // 先にトランスフォーム情報を取得
    let x;
    let y;
    
    if let Some(transform) = world.get_component::<Transform>(stock_id) {
        x = transform.position.x;
        y = transform.position.y;
    } else {
        return Err(JsValue::from_str("ストックのトランスフォームが見つかりません"));
    }
    
    // 全てのカードをストックに追加
    for (i, &card_id) in cards.iter().enumerate() {
        // カードの位置を設定
        card::set_card_position(world, card_id, x, y, i as i32)?;
    }
    
    // 別のスコープでスタックコンテナを取得して更新
    {
        if let Some(stock) = world.get_component_mut::<StackContainer>(stock_id) {
            // カードをストックに追加
            for &card_id in cards.iter() {
                stock.add_card(card_id);
            }
        }
    }
    
    Ok(())
}

/// ストックからウェイストにカードを移動
pub fn draw_from_stock(
    world: &mut World,
    stock_id: EntityId,
    waste_id: EntityId,
) -> Result<bool, JsValue> {
    // ストックを確認し、空の場合はウェイストからストックにカードを戻す
    let is_stock_empty = {
        let stock = match world.get_component::<StackContainer>(stock_id) {
            Some(stack) => stack,
            None => return Err(JsValue::from_str("ストックが見つかりません")),
        };
        
        stock.is_empty()
    };
    
    // ストックが空の場合
    if is_stock_empty {
        // ウェイストからストックにカードを戻す
        return reset_stock_from_waste(world, stock_id, waste_id);
    }
    
    // ストックの一番上のカードを取得
    let card_id = {
        let stock = match world.get_component::<StackContainer>(stock_id) {
            Some(stack) => stack,
            None => return Err(JsValue::from_str("ストックが見つかりません")),
        };
        
        match stock.top_card() {
            Some(id) => id,
            None => return Err(JsValue::from_str("ストックにカードがありません")),
        }
    };
    
    // ストックからカードを削除
    {
        if let Some(stock) = world.get_component_mut::<StackContainer>(stock_id) {
            stock.remove_card(card_id);
        }
    }
    
    // ウェイストの位置を取得
    let (waste_x, waste_y, waste_card_count) = {
        let waste_transform = match world.get_component::<Transform>(waste_id) {
            Some(transform) => transform,
            None => return Err(JsValue::from_str("ウェイストのトランスフォームが見つかりません")),
        };
        
        let waste = match world.get_component::<StackContainer>(waste_id) {
            Some(stack) => stack,
            None => return Err(JsValue::from_str("ウェイストが見つかりません")),
        };
        
        (waste_transform.position.x, waste_transform.position.y, waste.card_count() as i32)
    };
    
    // カードを表向きにする
    card::flip_card(world, card_id)?;
    
    // カードの位置を設定
    card::set_card_position(world, card_id, waste_x, waste_y, waste_card_count)?;
    
    // ウェイストにカードを追加
    {
        if let Some(waste) = world.get_component_mut::<StackContainer>(waste_id) {
            waste.add_card(card_id);
        }
    }
    
    Ok(true)
}

/// ウェイストからストックにカードを戻す
pub fn reset_stock_from_waste(
    world: &mut World,
    stock_id: EntityId,
    waste_id: EntityId,
) -> Result<bool, JsValue> {
    // ウェイストのコンテナを取得し、カードをコピー
    let waste_cards = {
        let waste = match world.get_component::<StackContainer>(waste_id) {
            Some(stack) => stack.clone(),
            None => return Err(JsValue::from_str("ウェイストが見つかりません")),
        };
        
        // ウェイストが空の場合は何もしない
        if waste.is_empty() {
            return Ok(false);
        }
        
        waste.cards.clone()
    };
    
    // ストックの位置を取得
    let (stock_x, stock_y) = {
        let stock_transform = match world.get_component::<Transform>(stock_id) {
            Some(transform) => transform.clone(),
            None => return Err(JsValue::from_str("ストックが見つかりません")),
        };
        
        (stock_transform.position.x, stock_transform.position.y)
    };
    
    // ウェイストからカードを削除
    {
        if let Some(waste) = world.get_component_mut::<StackContainer>(waste_id) {
            waste.cards.clear();
        }
    }
    
    // カードを裏向きにしてストックに追加
    for (i, &card_id) in waste_cards.iter().enumerate() {
        // カードを裏向きにする
        {
            if let Some(card_info) = world.get_component_mut::<CardInfo>(card_id) {
                card_info.face_up = false;
            }
        }
        
        // カードの位置を更新
        card::set_card_position(world, card_id, stock_x, stock_y, i as i32)?;
    }
    
    // 最後にストックにカードを追加
    {
        if let Some(stock) = world.get_component_mut::<StackContainer>(stock_id) {
            for &card_id in waste_cards.iter() {
                stock.add_card(card_id);
            }
        }
    }
    
    Ok(true)
}

/// タブローからファウンデーションへカードを移動できるかチェック
pub fn can_move_to_foundation(
    world: &World,
    card_id: EntityId,
    foundation_id: EntityId,
) -> bool {
    // カード情報を取得
    let card_info = match world.get_component::<CardInfo>(card_id) {
        Some(info) => info,
        None => return false,
    };
    
    // カードが表向きかチェック
    if !card_info.face_up {
        return false;
    }
    
    // ファウンデーション情報を取得
    let foundation = match world.get_component::<StackContainer>(foundation_id) {
        Some(stack) => stack,
        None => return false,
    };
    
    // ファウンデーションのスートを取得
    let foundation_suit = match foundation.stack_type {
        StackType::Foundation { suit } => suit as u8,
        _ => return false,
    };
    
    // ファウンデーションが空の場合
    if foundation.is_empty() {
        // Aのみ置ける
        return card_info.suit == foundation_suit && card_info.rank == 0;
    }
    
    // ファウンデーションの一番上のカードを取得
    let top_card_id = match foundation.top_card() {
        Some(id) => id,
        None => return false,
    };
    
    // 一番上のカード情報を取得
    let top_card_info = match world.get_component::<CardInfo>(top_card_id) {
        Some(info) => info,
        None => return false,
    };
    
    // スートが一致し、ランクが1つ上のカードのみ置ける
    card_info.suit == top_card_info.suit && card_info.rank == top_card_info.rank + 1
}

/// ゲームがクリアされたかチェック
pub fn check_game_clear(world: &World, foundation_ids: &[EntityId]) -> bool {
    // 全てのファウンデーションにK（ランク12）があるかチェック
    for &foundation_id in foundation_ids {
        let foundation = match world.get_component::<StackContainer>(foundation_id) {
            Some(stack) => stack,
            None => return false,
        };
        
        // カードが13枚（A~K）あるかチェック
        if foundation.card_count() != 13 {
            return false;
        }
        
        // 一番上のカードがKかチェック
        if let Some(top_card_id) = foundation.top_card() {
            if let Some(card_info) = world.get_component::<CardInfo>(top_card_id) {
                if card_info.rank != 12 {  // Kはランク12
                    return false;
                }
            } else {
                return false;
            }
        } else {
            return false;
        }
    }
    
    true
} 