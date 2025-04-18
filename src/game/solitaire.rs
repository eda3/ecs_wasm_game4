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
                
                // 表向きのカードが正しくドラッグ可能になっているか確認
                if let Some(card_info) = world.get_component::<CardInfo>(card_id) {
                    if card_info.face_up {
                        // 明示的にドラッグ可能にする
                        card::set_card_draggable(world, card_id, true)?;
                        info!("🃏 カードID: {} を表向きにしてドラッグ可能に設定しました", card_id);
                    }
                }
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
    
    // ストックが空の場合、ウェイストからカードを戻す
    if is_stock_empty {
        return reset_stock_from_waste(world, stock_id, waste_id);
    }
    
    // ストックから1枚取り出す
    let card_id = {
        let stock = match world.get_component_mut::<StackContainer>(stock_id) {
            Some(stack) => stack,
            None => return Err(JsValue::from_str("ストックが見つかりません")),
        };
        
        // カードがない場合は早期リターン
        if stock.is_empty() {
            return Ok(false);
        }
        
        // 最後のカードを取得
        stock.remove_top_card().ok_or_else(|| JsValue::from_str("カードの取得に失敗しました"))?
    };
    
    // ウェイストの位置情報を取得
    let waste_x;
    let waste_y;
    
    if let Some(transform) = world.get_component::<Transform>(waste_id) {
        waste_x = transform.position.x;
        waste_y = transform.position.y;
    } else {
        return Err(JsValue::from_str("ウェイストのトランスフォームが見つかりません"));
    }
    
    // ウェイストの現在のカード数を取得
    let waste_z_index = {
        let waste = match world.get_component::<StackContainer>(waste_id) {
            Some(stack) => stack,
            None => return Err(JsValue::from_str("ウェイストが見つかりません")),
        };
        
        waste.card_count() as i32
    };
    
    // カードをウェイストに移動
    card::set_card_position(world, card_id, waste_x, waste_y, waste_z_index)?;
    
    // カードを表向きにする
    card::flip_card(world, card_id)?;
    
    // 表向きになったカードを明示的にドラッグ可能に設定
    card::set_card_draggable(world, card_id, true)?;
    info!("🃏 ストックからウェイストに移動したカードID: {} をドラッグ可能に設定しました", card_id);
    
    // ウェイストにカードを追加
    if let Some(waste) = world.get_component_mut::<StackContainer>(waste_id) {
        waste.add_card(card_id);
    }
    
    Ok(true)
}

/// ウェイストからストックへカードを戻す
pub fn reset_stock_from_waste(
    world: &mut World,
    stock_id: EntityId,
    waste_id: EntityId,
) -> Result<bool, JsValue> {
    // ウェイストからカードを取得
    let waste_cards = {
        let waste = match world.get_component_mut::<StackContainer>(waste_id) {
            Some(stack) => stack,
            None => return Err(JsValue::from_str("ウェイストが見つかりません")),
        };
        
        // ウェイストが空の場合は早期リターン
        if waste.is_empty() {
            return Ok(false);
        }
        
        // 全てのカードを取得して、ウェイストをクリア
        let cards = waste.get_all_cards();
        waste.clear_cards();
        cards
    };
    
    // ストックの位置情報を取得
    let stock_x;
    let stock_y;
    
    if let Some(transform) = world.get_component::<Transform>(stock_id) {
        stock_x = transform.position.x;
        stock_y = transform.position.y;
    } else {
        return Err(JsValue::from_str("ストックのトランスフォームが見つかりません"));
    }
    
    // カードをストックに戻す（順番は逆順に）
    for (i, card_id) in waste_cards.iter().enumerate() {
        // カードを裏向きにする
        if let Some(card_info) = world.get_component_mut::<CardInfo>(*card_id) {
            card_info.face_up = false;
        }
        
        // カードの位置を設定
        card::set_card_position(world, *card_id, stock_x, stock_y, i as i32)?;
    }
    
    // ストックにカードを追加
    if let Some(stock) = world.get_component_mut::<StackContainer>(stock_id) {
        for card_id in waste_cards {
            stock.add_card(card_id);
        }
    }
    
    Ok(true)
}

/// カードをファウンデーションに移動できるかチェック
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
    
    // 裏向きのカードは移動できない
    if !card_info.face_up {
        return false;
    }
    
    // ファウンデーション情報を取得
    let foundation = match world.get_component::<StackContainer>(foundation_id) {
        Some(stack) => stack,
        None => return false,
    };
    
    // ファウンデーションのタイプをチェック
    if let StackType::Foundation { suit } = foundation.stack_type {
        // ファウンデーションが空の場合
        if foundation.is_empty() {
            // エースのみ置ける
            return card_info.rank == 0 && card_info.suit == suit as u8;
        } else {
            // 最上部のカードを取得
            let top_card_id = match foundation.get_top_card() {
                Some(id) => id,
                None => return false,
            };
            
            // 最上部のカード情報を取得
            let top_card_info = match world.get_component::<CardInfo>(top_card_id) {
                Some(info) => info,
                None => return false,
            };
            
            // 同じスートで連続するランクのみ置ける
            return card_info.suit == suit as u8 && card_info.rank == top_card_info.rank + 1;
        }
    }
    
    false
}

/// タブローにカードを移動できるかチェック
pub fn can_move_to_tableau(
    world: &World,
    card_id: EntityId,
    tableau_id: EntityId,
) -> bool {
    // カード情報を取得
    let card_info = match world.get_component::<CardInfo>(card_id) {
        Some(info) => info,
        None => return false,
    };
    
    // 裏向きのカードは移動できない
    if !card_info.face_up {
        return false;
    }
    
    // タブロー情報を取得
    let tableau = match world.get_component::<StackContainer>(tableau_id) {
        Some(stack) => stack,
        None => return false,
    };
    
    // タブローのタイプをチェック
    if let StackType::Tableau { .. } = tableau.stack_type {
        // タブローが空の場合
        if tableau.is_empty() {
            // キングのみ置ける
            return card_info.rank == 12;
        } else {
            // 最上部のカードを取得
            let top_card_id = match tableau.get_top_card() {
                Some(id) => id,
                None => return false,
            };
            
            // 最上部のカードとスタック可能かチェック
            return card::can_stack_card(world, card_id, top_card_id);
        }
    }
    
    false
}

/// カードを移動する
pub fn move_card(
    world: &mut World,
    card_id: EntityId,
    from_stack_id: EntityId,
    to_stack_id: EntityId,
) -> Result<bool, JsValue> {
    // 移動元のスタック情報を確認
    let can_remove = {
        let from_stack = match world.get_component::<StackContainer>(from_stack_id) {
            Some(stack) => stack,
            None => return Err(JsValue::from_str("移動元のスタックが見つかりません")),
        };
        
        // カードがこのスタックにあるか確認
        from_stack.contains_card(card_id)
    };
    
    if !can_remove {
        return Ok(false);
    }
    
    // 移動先の情報を取得
    let (to_x, to_y, to_z_index) = {
        // トランスフォーム情報を取得
        let transform = match world.get_component::<Transform>(to_stack_id) {
            Some(t) => t,
            None => return Err(JsValue::from_str("移動先のトランスフォームが見つかりません")),
        };
        
        // スタック情報を取得
        let to_stack = match world.get_component::<StackContainer>(to_stack_id) {
            Some(stack) => stack,
            None => return Err(JsValue::from_str("移動先のスタックが見つかりません")),
        };
        
        // 移動先の座標とZ-indexを計算
        let x = transform.position.x;
        let y = transform.position.y;
        let z = to_stack.card_count() as i32;
        
        (x, y, z)
    };
    
    // 移動元からカードを削除
    {
        let from_stack = match world.get_component_mut::<StackContainer>(from_stack_id) {
            Some(stack) => stack,
            None => return Err(JsValue::from_str("移動元のスタックが見つかりません")),
        };
        
        from_stack.remove_card(card_id);
    }
    
    // カードの位置を更新
    card::set_card_position(world, card_id, to_x, to_y, to_z_index)?;
    
    // 移動先にカードを追加
    {
        let to_stack = match world.get_component_mut::<StackContainer>(to_stack_id) {
            Some(stack) => stack,
            None => return Err(JsValue::from_str("移動先のスタックが見つかりません")),
        };
        
        to_stack.add_card(card_id);
    }
    
    // 移動元の最上部のカードを表向きにする
    // タブローの場合のみ行う
    {
        let from_stack = match world.get_component::<StackContainer>(from_stack_id) {
            Some(stack) => stack,
            None => return Err(JsValue::from_str("移動元のスタックが見つかりません")),
        };
        
        if let StackType::Tableau { .. } = from_stack.stack_type {
            if !from_stack.is_empty() {
                let top_card_id = from_stack.get_top_card().unwrap();
                let top_card_info = world.get_component::<CardInfo>(top_card_id);
                
                if let Some(card_info) = top_card_info {
                    if !card_info.face_up {
                        card::flip_card(world, top_card_id)?;
                    }
                }
            }
        }
    }
    
    Ok(true)
}

/// ファウンデーションを確認してゲームクリアを判定
pub fn check_game_clear(world: &World, foundation_ids: &[EntityId]) -> bool {
    // 全てのファウンデーションが埋まっているかチェック
    for &foundation_id in foundation_ids {
        // ファウンデーション情報を取得
        let foundation = match world.get_component::<StackContainer>(foundation_id) {
            Some(stack) => stack,
            None => return false,
        };
        
        // スタックのタイプを確認
        if let StackType::Foundation { .. } = foundation.stack_type {
            // 各ファウンデーションには13枚のカードがあるはず
            if foundation.card_count() != 13 {
                return false;
            }
            
            // 最上部のカードがKingか確認
            if let Some(top_card_id) = foundation.get_top_card() {
                if let Some(card_info) = world.get_component::<CardInfo>(top_card_id) {
                    if card_info.rank != 12 { // Kingのランクは12
                        return false;
                    }
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }
    }
    
    true
}

/// タブローのカードをファウンデーションに自動的に移動する
pub fn auto_complete(
    world: &mut World,
    tableau_ids: &[EntityId],
    foundation_ids: &[EntityId],
    waste_id: EntityId,
) -> Result<bool, JsValue> {
    let mut moved_any_card = false;
    
    // タブローの各列から移動可能なカードを検索
    for &tableau_id in tableau_ids {
        if let Some(tableau) = world.get_component::<StackContainer>(tableau_id) {
            if tableau.is_empty() {
                continue;
            }
            
            let top_card_id = tableau.get_top_card().unwrap();
            
            // 各ファウンデーションに移動できるか確認
            for &foundation_id in foundation_ids {
                if can_move_to_foundation(world, top_card_id, foundation_id) {
                    // 移動可能ならカードを移動
                    move_card(world, top_card_id, tableau_id, foundation_id)?;
                    moved_any_card = true;
                    break;
                }
            }
        }
    }
    
    // ウェイストからファウンデーションへの移動
    if let Some(waste) = world.get_component::<StackContainer>(waste_id) {
        if !waste.is_empty() {
            let top_card_id = waste.get_top_card().unwrap();
            
            // 各ファウンデーションに移動できるか確認
            for &foundation_id in foundation_ids {
                if can_move_to_foundation(world, top_card_id, foundation_id) {
                    // 移動可能ならカードを移動
                    move_card(world, top_card_id, waste_id, foundation_id)?;
                    moved_any_card = true;
                    break;
                }
            }
        }
    }
    
    Ok(moved_any_card)
}

/// カードまたはカードのスタックを移動
pub fn move_card_stack(
    world: &mut World,
    card_id: EntityId,
    from_stack_id: EntityId,
    to_stack_id: EntityId,
) -> Result<bool, JsValue> {
    // 移動元のスタック情報を確認
    let (from_stack_type, from_cards) = {
        let from_stack = match world.get_component::<StackContainer>(from_stack_id) {
            Some(stack) => stack,
            None => return Err(JsValue::from_str("移動元のスタックが見つかりません")),
        };
        
        // カードがこのスタックにあるか確認
        if !from_stack.contains_card(card_id) {
            return Ok(false);
        }
        
        // カードのインデックスを見つける
        let card_index = from_stack.get_card_index(card_id).unwrap();
        
        // このカードから最後までのカードを取得
        let cards = from_stack.get_all_cards()[card_index..].to_vec();
        
        (from_stack.stack_type, cards)
    };

    // 移動先のスタック情報を確認
    let (can_move_stack, to_stack_type) = {
        let to_stack = match world.get_component::<StackContainer>(to_stack_id) {
            Some(stack) => stack,
            None => return Err(JsValue::from_str("移動先のスタックが見つかりません")),
        };
        
        let can_move = match (from_stack_type, to_stack.stack_type) {
            // タブローからタブローへの移動
            (StackType::Tableau { .. }, StackType::Tableau { .. }) => {
                // 空のタブローへは最初のカードがKingでないと置けない
                if to_stack.is_empty() {
                    let first_card_info = world.get_component::<CardInfo>(from_cards[0]);
                    if let Some(card_info) = first_card_info {
                        card_info.rank == 12 // Kingのランクは12
                    } else {
                        false
                    }
                } else {
                    // 最上部のカードを取得
                    let top_card_id = to_stack.get_top_card().unwrap();
                    
                    // 最初のカードが最上部のカードにスタック可能か確認
                    card::can_stack_card(world, from_cards[0], top_card_id)
                }
            },
            // タブロー以外からファウンデーションへの移動（1枚だけ）
            (_, StackType::Foundation { .. }) => {
                // 複数のカードは移動できない
                if from_cards.len() > 1 {
                    false
                } else {
                    can_move_to_foundation(world, from_cards[0], to_stack_id)
                }
            },
            // その他の移動（基本的には1枚ずつ）
            _ => from_cards.len() == 1 && can_move_to_tableau(world, from_cards[0], to_stack_id),
        };
        
        (can_move, to_stack.stack_type)
    };
    
    if !can_move_stack {
        return Ok(false);
    }
    
    // 移動元からカードを削除
    {
        let from_stack = match world.get_component_mut::<StackContainer>(from_stack_id) {
            Some(stack) => stack,
            None => return Err(JsValue::from_str("移動元のスタックが見つかりません")),
        };
        
        // 移動するカードを全て削除
        for &card in &from_cards {
            from_stack.remove_card(card);
        }
    }
    
    // 移動先の位置情報を取得
    let (base_x, base_y, current_card_count) = {
        // トランスフォーム情報を取得
        let transform = match world.get_component::<Transform>(to_stack_id) {
            Some(t) => t,
            None => return Err(JsValue::from_str("移動先のトランスフォームが見つかりません")),
        };
        
        // スタック情報を取得
        let to_stack = match world.get_component::<StackContainer>(to_stack_id) {
            Some(stack) => stack,
            None => return Err(JsValue::from_str("移動先のスタックが見つかりません")),
        };
        
        // 移動先の座標とカード数を計算
        (transform.position.x, transform.position.y, to_stack.card_count())
    };
    
    // カードを移動して追加
    for (i, &card) in from_cards.iter().enumerate() {
        // 位置とZ-indexを計算
        let z_index = (current_card_count + i) as i32;
        let y_offset = if let StackType::Tableau { .. } = to_stack_type {
            i as f64 * crate::constants::STACK_OFFSET_Y
        } else {
            0.0
        };
        
        // カードの位置を更新
        card::set_card_position(world, card, base_x, base_y + y_offset, z_index)?;
    }
    
    // 移動先にカードを追加
    {
        let to_stack = match world.get_component_mut::<StackContainer>(to_stack_id) {
            Some(stack) => stack,
            None => return Err(JsValue::from_str("移動先のスタックが見つかりません")),
        };
        
        // カードを追加
        for &card in &from_cards {
            to_stack.add_card(card);
        }
    }
    
    // 移動元の最上部のカードを表向きにする
    // タブローの場合のみ行う
    {
        let from_stack = match world.get_component::<StackContainer>(from_stack_id) {
            Some(stack) => stack,
            None => return Err(JsValue::from_str("移動元のスタックが見つかりません")),
        };
        
        if let StackType::Tableau { .. } = from_stack.stack_type {
            if !from_stack.is_empty() {
                let top_card_id = from_stack.get_top_card().unwrap();
                let top_card_info = world.get_component::<CardInfo>(top_card_id);
                
                if let Some(card_info) = top_card_info {
                    if !card_info.face_up {
                        card::flip_card(world, top_card_id)?;
                    }
                }
            }
        }
    }
    
    Ok(true)
}

/// リプレイ可能かつ解ける状態かチェック
pub fn is_winnable(world: &World) -> bool {
    // 全てのカードが表向きになっているか確認
    let all_cards_face_up = world.get_entities_with_component::<CardInfo>()
        .iter()
        .all(|&card_id| {
            if let Some(card_info) = world.get_component::<CardInfo>(card_id) {
                card_info.face_up
            } else {
                false
            }
        });
    
    // もし全てのカードが表向きなら、理論的には解ける
    all_cards_face_up
} 