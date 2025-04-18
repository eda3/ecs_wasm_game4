use std::collections::HashMap;
use std::any::{Any, TypeId};
use wasm_bindgen::prelude::*;
use crate::ecs::entity::EntityId;
use crate::utils::Vec2;

/// コンポーネントのデータを格納するためのトレイト
/// 任意の型をコンポーネントとして使用可能にする
pub trait Component: 'static {
    /// コンポーネントの名前を返す
    /// デバッグ用
    fn name(&self) -> &'static str;
}

/// コンポーネントストレージ
/// 特定の型のコンポーネントを複数のエンティティに対して保存する
pub struct ComponentStorage<T: Component> {
    // エンティティIDからコンポーネントへのマップ
    components: HashMap<EntityId, T>,
}

impl<T: Component> ComponentStorage<T> {
    /// 新しいコンポーネントストレージを作成
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }
    
    /// エンティティにコンポーネントを追加
    pub fn add(&mut self, entity_id: EntityId, component: T) {
        self.components.insert(entity_id, component);
    }
    
    /// エンティティからコンポーネントを削除
    pub fn remove(&mut self, entity_id: &EntityId) -> Option<T> {
        self.components.remove(entity_id)
    }
    
    /// エンティティのコンポーネントへの参照を取得
    pub fn get(&self, entity_id: &EntityId) -> Option<&T> {
        self.components.get(entity_id)
    }
    
    /// エンティティのコンポーネントへの可変参照を取得
    pub fn get_mut(&mut self, entity_id: &EntityId) -> Option<&mut T> {
        self.components.get_mut(entity_id)
    }
    
    /// エンティティがこのタイプのコンポーネントを持っているかチェック
    pub fn has(&self, entity_id: &EntityId) -> bool {
        self.components.contains_key(entity_id)
    }
    
    /// 全コンポーネントへの参照のイテレータを返す
    pub fn iter(&self) -> impl Iterator<Item = (&EntityId, &T)> {
        self.components.iter()
    }
    
    /// 全コンポーネントへの可変参照のイテレータを返す
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&EntityId, &mut T)> {
        self.components.iter_mut()
    }
    
    /// 指定したエンティティを削除
    pub fn remove_entity(&mut self, entity_id: &EntityId) {
        self.components.remove(entity_id);
    }
    
    /// 全てのコンポーネントを削除
    pub fn clear(&mut self) {
        self.components.clear();
    }
}

/// コンポーネントマネージャー
/// 全ての型のコンポーネントストレージを管理する
pub struct ComponentManager {
    // TypeIdからAny型へのマップ
    // 各ComponentStorage<T>はAny型としてダウンキャストできる
    storages: HashMap<TypeId, Box<dyn Any>>,
}

impl ComponentManager {
    /// 新しいコンポーネントマネージャーを作成
    pub fn new() -> Self {
        Self {
            storages: HashMap::new(),
        }
    }
    
    /// 指定した型のコンポーネントストレージを取得または作成
    fn get_or_create_storage<T: Component>(&mut self) -> &mut ComponentStorage<T> {
        let type_id = TypeId::of::<T>();
        
        if !self.storages.contains_key(&type_id) {
            let storage = ComponentStorage::<T>::new();
            self.storages.insert(type_id, Box::new(storage));
        }
        
        self.storages
            .get_mut(&type_id)
            .unwrap()
            .downcast_mut::<ComponentStorage<T>>()
            .unwrap()
    }
    
    /// コンポーネントストレージへの参照を取得
    fn get_storage<T: Component>(&self) -> Option<&ComponentStorage<T>> {
        let type_id = TypeId::of::<T>();
        
        self.storages
            .get(&type_id)
            .and_then(|boxed| boxed.downcast_ref::<ComponentStorage<T>>())
    }
    
    /// コンポーネントストレージへの可変参照を取得
    fn get_storage_mut<T: Component>(&mut self) -> Option<&mut ComponentStorage<T>> {
        let type_id = TypeId::of::<T>();
        
        self.storages
            .get_mut(&type_id)
            .and_then(|boxed| boxed.downcast_mut::<ComponentStorage<T>>())
    }
    
    /// エンティティにコンポーネントを追加
    pub fn add_component<T: Component>(&mut self, entity_id: EntityId, component: T) {
        let storage = self.get_or_create_storage::<T>();
        storage.add(entity_id, component);
    }
    
    /// エンティティからコンポーネントを削除
    pub fn remove_component<T: Component>(&mut self, entity_id: &EntityId) -> Option<T> {
        self.get_storage_mut::<T>()
            .and_then(|storage| storage.remove(entity_id))
    }
    
    /// エンティティのコンポーネントへの参照を取得
    pub fn get_component<T: Component>(&self, entity_id: &EntityId) -> Option<&T> {
        self.get_storage::<T>()
            .and_then(|storage| storage.get(entity_id))
    }
    
    /// エンティティのコンポーネントへの可変参照を取得
    pub fn get_component_mut<T: Component>(&mut self, entity_id: &EntityId) -> Option<&mut T> {
        self.get_storage_mut::<T>()
            .and_then(|storage| storage.get_mut(entity_id))
    }
    
    /// エンティティがコンポーネントを持っているかチェック
    pub fn has_component<T: Component>(&self, entity_id: &EntityId) -> bool {
        self.get_storage::<T>()
            .map_or(false, |storage| storage.has(entity_id))
    }
    
    /// 特定の型のコンポーネントを持つ全てのエンティティIDのイテレータを返す
    pub fn entities_with_component<T: Component>(&self) -> Vec<EntityId> {
        if let Some(storage) = self.get_storage::<T>() {
            storage.iter().map(|(entity_id, _)| *entity_id).collect()
        } else {
            Vec::new()
        }
    }
    
    /// エンティティに関連付けられたすべてのコンポーネントを削除
    pub fn remove_entity(&mut self, entity_id: &EntityId) {
        for (_type_id, storage) in self.storages.iter_mut() {
            // 各ストレージタイプに対してエンティティを削除するメソッドを呼び出す
            // Any型なので実行時に型を判断して適切なメソッドを呼ぶ必要がある
            // これは少し複雑なので、以下のようなヘルパーを作る
            remove_entity_from_storage(storage.as_mut(), entity_id);
        }
    }
    
    /// 全てのコンポーネントを削除
    pub fn clear(&mut self) {
        for (_type_id, storage) in self.storages.iter_mut() {
            clear_storage(storage.as_mut());
        }
    }
}

// ヘルパー関数：Any型のストレージからエンティティを削除
fn remove_entity_from_storage(storage: &mut dyn Any, entity_id: &EntityId) {
    // ダウンキャストして、特定の型のComponentStorageとして処理
    macro_rules! try_downcast_and_remove {
        ($type:ty) => {
            if let Some(typed_storage) = storage.downcast_mut::<ComponentStorage<$type>>() {
                typed_storage.remove_entity(entity_id);
                return;
            }
        };
    }
    
    // サポートする全てのコンポーネント型に対してダウンキャストを試みる
    try_downcast_and_remove!(Transform);
    try_downcast_and_remove!(CardInfo);
    try_downcast_and_remove!(Renderable);
    try_downcast_and_remove!(Draggable);
    try_downcast_and_remove!(Clickable);
    try_downcast_and_remove!(StackContainer);
    try_downcast_and_remove!(Position);
    try_downcast_and_remove!(Sprite);
    try_downcast_and_remove!(Droppable);
}

// ヘルパー関数：Any型のストレージをクリア
fn clear_storage(storage: &mut dyn Any) {
    // ダウンキャストして、特定の型のComponentStorageとして処理
    macro_rules! try_downcast_and_clear {
        ($type:ty) => {
            if let Some(typed_storage) = storage.downcast_mut::<ComponentStorage<$type>>() {
                typed_storage.clear();
                return;
            }
        };
    }
    
    // サポートする全てのコンポーネント型に対してダウンキャストを試みる
    try_downcast_and_clear!(Transform);
    try_downcast_and_clear!(CardInfo);
    try_downcast_and_clear!(Renderable);
    try_downcast_and_clear!(Draggable);
    try_downcast_and_clear!(Clickable);
    try_downcast_and_clear!(StackContainer);
    try_downcast_and_clear!(Position);
    try_downcast_and_clear!(Sprite);
    try_downcast_and_clear!(Droppable);
}

//
// 以下、ゲームで使用する各種コンポーネントの定義
//

/// トランスフォームコンポーネント
/// エンティティの位置、スケール、回転などを管理
#[derive(Clone, Debug)]
pub struct Transform {
    pub position: Vec2,
    pub scale: Vec2,
    pub rotation: f64,    // ラジアン単位の回転
    pub z_index: i32,     // 描画順序（大きいほど前面）
}

impl Transform {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            position: Vec2::new(x, y),
            scale: Vec2::new(1.0, 1.0),
            rotation: 0.0,
            z_index: 0,
        }
    }
    
    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }
}

impl Component for Transform {
    fn name(&self) -> &'static str {
        "Transform"
    }
}

/// カード情報コンポーネント
/// トランプカードの情報（スート、数字、表裏など）を管理
#[derive(Clone, Debug)]
pub struct CardInfo {
    pub suit: u8,        // 0=ハート, 1=ダイヤ, 2=クラブ, 3=スペード
    pub rank: u8,        // 0=A, 1=2, ..., 12=K
    pub face_up: bool,   // true=表向き, false=裏向き
    pub color: u8,       // 0=赤, 1=黒
}

impl CardInfo {
    pub fn new(suit: u8, rank: u8) -> Self {
        // 色の計算: ハートとダイヤは赤(0)、クラブとスペードは黒(1)
        let color = if suit < 2 { 0 } else { 1 };
        
        Self {
            suit,
            rank,
            face_up: false,
            color,
        }
    }
    
    pub fn face_up(mut self) -> Self {
        self.face_up = true;
        self
    }
    
    pub fn is_red(&self) -> bool {
        self.color == 0
    }
    
    pub fn is_black(&self) -> bool {
        self.color == 1
    }
    
    pub fn get_symbol(&self) -> &'static str {
        use crate::constants::{SUIT_SYMBOLS, RANK_SYMBOLS};
        
        RANK_SYMBOLS[self.rank as usize]
    }
    
    pub fn get_suit_symbol(&self) -> &'static str {
        use crate::constants::SUIT_SYMBOLS;
        
        SUIT_SYMBOLS[self.suit as usize]
    }
}

impl Component for CardInfo {
    fn name(&self) -> &'static str {
        "CardInfo"
    }
}

/// レンダラブルコンポーネント
/// エンティティの描画方法を定義
#[derive(Clone, Debug)]
pub struct Renderable {
    pub width: f64,
    pub height: f64,
    pub visible: bool,
    pub opacity: f64,
    pub render_type: RenderType,
}

/// レンダリングタイプ
/// エンティティの表示方法を指定
#[derive(Clone, Debug)]
pub enum RenderType {
    // カードの描画情報
    Card,
    // テキストの描画情報
    Text {
        text: String,
        font: String,
        color: String,
        align: String,
        baseline: String,
    },
    // 長方形の描画情報
    Rectangle {
        fill_color: String,
        stroke_color: String,
        stroke_width: f64,
        corner_radius: f64,
    },
    // カスタム描画関数（将来の拡張用）
    Custom,
}

impl Renderable {
    pub fn card(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            visible: true,
            opacity: 1.0,
            render_type: RenderType::Card,
        }
    }
    
    pub fn rectangle(
        width: f64,
        height: f64,
        fill_color: &str,
        stroke_color: &str,
        stroke_width: f64,
        corner_radius: f64,
    ) -> Self {
        Self {
            width,
            height,
            visible: true,
            opacity: 1.0,
            render_type: RenderType::Rectangle {
                fill_color: fill_color.to_string(),
                stroke_color: stroke_color.to_string(),
                stroke_width,
                corner_radius,
            },
        }
    }
    
    pub fn text(
        width: f64,
        height: f64,
        text: &str,
        font: &str,
        color: &str,
        align: &str,
        baseline: &str,
    ) -> Self {
        Self {
            width,
            height,
            visible: true,
            opacity: 1.0,
            render_type: RenderType::Text {
                text: text.to_string(),
                font: font.to_string(),
                color: color.to_string(),
                align: align.to_string(),
                baseline: baseline.to_string(),
            },
        }
    }
}

impl Component for Renderable {
    fn name(&self) -> &'static str {
        "Renderable"
    }
}

/// ドラッグ可能コンポーネント
/// エンティティをドラッグ可能にする
#[derive(Clone, Debug)]
pub struct Draggable {
    pub is_dragging: bool,
    pub drag_offset: Vec2,  // ドラッグ開始位置からのオフセット
    pub original_position: Vec2,  // ドラッグ開始前の位置
    pub original_z_index: i32,  // ドラッグ開始前のZ-index
    pub width: f64,  // ドラッグ可能な領域の幅
    pub height: f64,  // ドラッグ可能な領域の高さ
    pub drag_children: bool,  // 子要素も一緒にドラッグするか
}

impl Draggable {
    pub fn new() -> Self {
        Self {
            is_dragging: false,
            drag_offset: Vec2::zero(),
            original_position: Vec2::zero(),
            original_z_index: 0,
            width: 0.0,
            height: 0.0,
            drag_children: false,
        }
    }
    
    pub fn with_size(mut self, width: f64, height: f64) -> Self {
        self.width = width;
        self.height = height;
        self
    }
    
    pub fn with_drag_children(mut self) -> Self {
        self.drag_children = true;
        self
    }
}

impl Component for Draggable {
    fn name(&self) -> &'static str {
        "Draggable"
    }
}

/// クリック可能コンポーネント
/// エンティティをクリック可能にする
#[derive(Clone, Debug)]
pub struct Clickable {
    pub is_hovering: bool,
    pub was_clicked: bool,
    pub click_handler: ClickHandlerType,
}

/// クリックハンドラータイプ
/// クリック時の動作を指定
#[derive(Clone, Debug)]
pub enum ClickHandlerType {
    /// カードをめくる動作
    FlipCard,
    /// ストックからカードを引く動作
    DrawFromStock,
    /// ウェイストからカードを引く動作
    DrawFromWaste,
    /// タブローからカードを引く動作
    DrawFromTableau { column: usize },
    /// ファウンデーションからカードを引く動作
    DrawFromFoundation { stack: usize },
    /// カスタム動作（将来の拡張用）
    Custom,
}

impl Clickable {
    pub fn new(handler: ClickHandlerType) -> Self {
        Self {
            is_hovering: false,
            was_clicked: false,
            click_handler: handler,
        }
    }
}

impl Component for Clickable {
    fn name(&self) -> &'static str {
        "Clickable"
    }
}

/// スタックコンテナコンポーネント
/// カードの山を表現するコンポーネント
#[derive(Clone, Debug)]
pub struct StackContainer {
    pub stack_type: StackType,
    pub cards: Vec<EntityId>,
    pub max_cards: Option<usize>,
}

/// スタックタイプ
/// カードの山の種類を指定
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StackType {
    /// 山札
    Stock,
    /// 捨て札
    Waste,
    /// 場札（下に広げて並べるカードの列）
    Tableau { column: usize },
    /// 組み札（同じ柄のA～Kを集める場所）
    Foundation { suit: usize },
    /// 手札（ドラッグ中の一時的なカードグループ）
    Hand,
}

impl StackContainer {
    pub fn new(stack_type: StackType) -> Self {
        let max_cards = match stack_type {
            StackType::Foundation { .. } => Some(13),  // A～K
            _ => None,
        };
        
        Self {
            stack_type,
            cards: Vec::new(),
            max_cards,
        }
    }
    
    /// カードを追加
    pub fn add_card(&mut self, card_id: EntityId) {
        self.cards.push(card_id);
    }
    
    /// カードを削除
    pub fn remove_card(&mut self, card_id: EntityId) -> bool {
        if let Some(index) = self.cards.iter().position(|id| *id == card_id) {
            self.cards.remove(index);
            true
        } else {
            false
        }
    }
    
    /// スタックの一番上のカードを削除して返す
    pub fn remove_top_card(&mut self) -> Option<EntityId> {
        self.cards.pop()
    }
    
    /// スタックの一番上のカードID（最後の要素）を返す
    pub fn top_card(&self) -> Option<EntityId> {
        self.cards.last().copied()
    }
    
    /// スタックの一番上のカードID（最後の要素）を返す（top_cardの別名）
    pub fn get_top_card(&self) -> Option<EntityId> {
        self.top_card()
    }
    
    /// スタック内のすべてのカードのリストを返す
    pub fn get_all_cards(&self) -> Vec<EntityId> {
        self.cards.clone()
    }
    
    /// スタック内のすべてのカードをクリア
    pub fn clear_cards(&mut self) {
        self.cards.clear();
    }
    
    /// 指定したカードのインデックスを返す
    pub fn get_card_index(&self, card_id: EntityId) -> Option<usize> {
        self.cards.iter().position(|id| *id == card_id)
    }
    
    /// スタックに指定したカードが含まれているかチェック
    pub fn contains_card(&self, card_id: EntityId) -> bool {
        self.cards.contains(&card_id)
    }
    
    /// スタックが空かどうか
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
    
    /// スタックのカード数
    pub fn card_count(&self) -> usize {
        self.cards.len()
    }
    
    /// 指定したインデックス以降のカードを取得
    pub fn cards_from_index(&self, index: usize) -> Vec<EntityId> {
        if index >= self.cards.len() {
            return Vec::new();
        }
        self.cards[index..].to_vec()
    }
    
    /// 指定したインデックス以降のカードを削除し、削除したカードを返す
    pub fn remove_cards_from_index(&mut self, index: usize) -> Vec<EntityId> {
        if index >= self.cards.len() {
            return Vec::new();
        }
        let removed_cards = self.cards[index..].to_vec();
        self.cards.truncate(index);
        removed_cards
    }
}

impl Component for StackContainer {
    fn name(&self) -> &'static str {
        "StackContainer"
    }
}

// 位置情報を表すコンポーネント
#[derive(Clone, Debug)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl Position {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl Component for Position {
    fn name(&self) -> &'static str {
        "Position"
    }
}

// スプライト表示用コンポーネント
#[derive(Clone, Debug)]
pub struct Sprite {
    pub width: f64,
    pub height: f64,
    pub color: String,
    pub image_key: String,
    pub visible: bool,
}

impl Sprite {
    pub fn new(width: f64, height: f64, color: &str) -> Self {
        Self {
            width,
            height,
            color: color.to_string(),
            image_key: String::new(),
            visible: true,
        }
    }
    
    pub fn with_image(mut self, image_key: &str) -> Self {
        self.image_key = image_key.to_string();
        self
    }
}

impl Component for Sprite {
    fn name(&self) -> &'static str {
        "Sprite"
    }
}

/// ドロップ可能なコンポーネント
/// エンティティをドロップ対象として指定する
#[derive(Clone, Debug)]
pub struct Droppable {
    pub width: f64,  // ドロップ可能な領域の幅
    pub height: f64,  // ドロップ可能な領域の高さ
    pub drop_types: Vec<usize>,  // 受け入れ可能なドラッグタイプ（将来の拡張用）
    pub is_active: bool,  // ドロップが現在有効かどうか
}

impl Droppable {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            drop_types: Vec::new(),
            is_active: true,
        }
    }
    
    pub fn with_drop_types(mut self, types: Vec<usize>) -> Self {
        self.drop_types = types;
        self
    }
    
    pub fn with_active(mut self, active: bool) -> Self {
        self.is_active = active;
        self
    }
}

impl Component for Droppable {
    fn name(&self) -> &'static str {
        "Droppable"
    }
} 