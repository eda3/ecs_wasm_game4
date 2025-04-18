use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d};
use crate::ecs::world::World;
use crate::ecs::resources::ResourceManager;
use crate::ecs::component::{Transform, Renderable, CardInfo, RenderType, Position, Sprite, Draggable};
use crate::constants::{
    CARD_WIDTH, CARD_HEIGHT, CARD_FRONT_COLOR, CARD_BACK_COLOR,
    CARD_BORDER_COLOR, CARD_TEXT_COLOR, CARD_RED_COLOR, CARD_BLACK_COLOR,
    CARD_BORDER_RADIUS,
};
use log::error;
use super::RenderContext;

/// レンダラー
/// キャンバスへの描画を担当
#[derive(Clone)]
pub struct Renderer {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
}

impl Renderer {
    /// 新しいレンダラーを作成
    pub fn new(canvas: HtmlCanvasElement, context: CanvasRenderingContext2d) -> Self {
        Self { canvas, context }
    }
    
    /// ゲーム世界を描画
    pub fn render(&self, world: &World, _resources: &ResourceManager) -> Result<(), JsValue> {
        // キャンバスをクリア
        self.clear_canvas()?;
        
        // エンティティをZ-indexでソート（描画順序のため）
        let mut entities_to_render: Vec<_> = world
            .get_all_entities()
            .iter()
            .filter_map(|&entity_id| {
                if let Some(transform) = world.get_component::<Transform>(entity_id) {
                    if world.get_component::<Renderable>(entity_id).is_some() {
                        Some((entity_id, transform.z_index))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        
        // Z-indexでソート（小さい順に描画）
        entities_to_render.sort_by_key(|&(_, z_index)| z_index);
        
        // 各エンティティを描画
        for (entity_id, _) in entities_to_render {
            self.render_entity(world, entity_id)?;
        }
        
        Ok(())
    }
    
    /// キャンバスをクリア
    fn clear_canvas(&self) -> Result<(), JsValue> {
        self.context.set_fill_style(&JsValue::from_str("#076324"));  // 緑色の背景（ソリティア風）
        self.context.fill_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );
        Ok(())
    }
    
    /// エンティティを描画
    fn render_entity(&self, world: &World, entity_id: usize) -> Result<(), JsValue> {
        // 必要なコンポーネントを取得
        let transform = match world.get_component::<Transform>(entity_id) {
            Some(t) => t,
            None => return Ok(()),
        };
        
        let renderable = match world.get_component::<Renderable>(entity_id) {
            Some(r) => r,
            None => return Ok(()),
        };
        
        // 非表示の場合は描画しない
        if !renderable.visible {
            return Ok(());
        }
        
        // コンテキストの状態を保存
        self.context.save();
        
        // 描画位置に移動
        self.context.translate(transform.position.x, transform.position.y)?;
        
        // 回転を適用
        if transform.rotation != 0.0 {
            self.context.rotate(transform.rotation)?;
        }
        
        // スケールを適用
        if transform.scale.x != 1.0 || transform.scale.y != 1.0 {
            self.context.scale(transform.scale.x, transform.scale.y)?;
        }
        
        // 不透明度を設定
        self.context.set_global_alpha(renderable.opacity);
        
        // レンダリングタイプに応じて描画
        match &renderable.render_type {
            RenderType::Card => {
                // カード情報を取得
                if let Some(card_info) = world.get_component::<CardInfo>(entity_id) {
                    self.render_card(card_info, renderable.width, renderable.height)?;
                } else {
                    // カード情報がない場合は単純な長方形を描画
                    self.render_rectangle(
                        renderable.width,
                        renderable.height,
                        CARD_BACK_COLOR,
                        CARD_BORDER_COLOR,
                        1.0,
                        CARD_BORDER_RADIUS,
                    )?;
                }
            },
            RenderType::Text {
                text,
                font,
                color,
                align,
                baseline,
            } => {
                self.render_text(
                    text,
                    font,
                    color,
                    align,
                    baseline,
                    renderable.width / 2.0,
                    renderable.height / 2.0,
                )?;
            },
            RenderType::Rectangle {
                fill_color,
                stroke_color,
                stroke_width,
                corner_radius,
            } => {
                self.render_rectangle(
                    renderable.width,
                    renderable.height,
                    fill_color,
                    stroke_color,
                    *stroke_width,
                    *corner_radius,
                )?;
            },
            RenderType::Custom => {
                // カスタム描画関数は実装しない（必要に応じて拡張）
            },
        }
        
        // コンテキストの状態を復元
        self.context.restore();
        
        Ok(())
    }
    
    /// カードを描画
    fn render_card(&self, card_info: &CardInfo, width: f64, height: f64) -> Result<(), JsValue> {
        // カードの裏表で描画方法を変える
        if card_info.face_up {
            // 表向きカードを描画
            self.render_rectangle(
                width,
                height,
                CARD_FRONT_COLOR,
                CARD_BORDER_COLOR,
                1.0,
                CARD_BORDER_RADIUS,
            )?;
            
            // スートに応じた色を設定
            let color = if card_info.is_red() {
                CARD_RED_COLOR
            } else {
                CARD_BLACK_COLOR
            };
            
            // 左上にランクとスート記号を描画
            let rank_text = card_info.get_symbol();
            let suit_symbol = card_info.get_suit_symbol();
            
            // 左上の小さなランク・スート記号
            self.context.set_font("16px Arial");
            self.context.set_fill_style(&JsValue::from_str(color));
            self.context.set_text_align("left");
            self.context.set_text_baseline("top");
            self.context.fill_text(&format!("{}{}", rank_text, suit_symbol), 5.0, 5.0)?;
            
            // 中央の大きなランク・スート記号
            self.context.set_font("32px Arial");
            self.context.set_text_align("center");
            self.context.set_text_baseline("middle");
            self.context.fill_text(&format!("{}{}", rank_text, suit_symbol), width / 2.0, height / 2.0)?;
            
            // 右下の小さなランク・スート記号（上下逆）
            self.context.save();
            self.context.translate(width, height)?;
            self.context.rotate(std::f64::consts::PI)?;
            self.context.set_font("16px Arial");
            self.context.set_text_align("left");
            self.context.set_text_baseline("top");
            self.context.fill_text(&format!("{}{}", rank_text, suit_symbol), 5.0, 5.0)?;
            self.context.restore();
            
        } else {
            // 裏向きカードを描画
            self.render_rectangle(
                width,
                height,
                CARD_BACK_COLOR,
                CARD_BORDER_COLOR,
                1.0,
                CARD_BORDER_RADIUS,
            )?;
            
            // カードの裏面パターンを描画
            self.context.set_stroke_style(&JsValue::from_str("#FFFFFF33"));
            self.context.set_line_width(2.0);
            
            // 格子パターン
            let gap = 10.0;
            for x in (gap as u32..width as u32).step_by(gap as usize) {
                self.context.begin_path();
                self.context.move_to(x as f64, 0.0);
                self.context.line_to(x as f64, height);
                self.context.stroke();
            }
            
            for y in (gap as u32..height as u32).step_by(gap as usize) {
                self.context.begin_path();
                self.context.move_to(0.0, y as f64);
                self.context.line_to(width, y as f64);
                self.context.stroke();
            }
        }
        
        Ok(())
    }
    
    /// 長方形を描画
    fn render_rectangle(
        &self,
        width: f64,
        height: f64,
        fill_color: &str,
        stroke_color: &str,
        stroke_width: f64,
        corner_radius: f64,
    ) -> Result<(), JsValue> {
        // 角丸長方形のパスを作成
        self.context.begin_path();
        self.context.move_to(corner_radius, 0.0);
        self.context.line_to(width - corner_radius, 0.0);
        self.context.arc_to(width, 0.0, width, corner_radius, corner_radius)?;
        self.context.line_to(width, height - corner_radius);
        self.context.arc_to(width, height, width - corner_radius, height, corner_radius)?;
        self.context.line_to(corner_radius, height);
        self.context.arc_to(0.0, height, 0.0, height - corner_radius, corner_radius)?;
        self.context.line_to(0.0, corner_radius);
        self.context.arc_to(0.0, 0.0, corner_radius, 0.0, corner_radius)?;
        self.context.close_path();
        
        // 塗りつぶし
        self.context.set_fill_style(&JsValue::from_str(fill_color));
        self.context.fill();
        
        // 枠線
        self.context.set_stroke_style(&JsValue::from_str(stroke_color));
        self.context.set_line_width(stroke_width);
        self.context.stroke();
        
        Ok(())
    }
    
    /// テキストを描画
    fn render_text(
        &self,
        text: &str,
        font: &str,
        color: &str,
        align: &str,
        baseline: &str,
        x: f64,
        y: f64,
    ) -> Result<(), JsValue> {
        self.context.set_font(font);
        self.context.set_fill_style(&JsValue::from_str(color));
        self.context.set_text_align(align);
        self.context.set_text_baseline(baseline);
        self.context.fill_text(text, x, y)?;
        
        Ok(())
    }
}

/// ゲームのレンダリングを担当するレンダラー
pub struct GameRenderer {
    context: RenderContext,
}

impl GameRenderer {
    /// 新しいレンダラーを作成
    pub fn new(canvas_id: &str) -> Result<Self, JsValue> {
        let context = RenderContext::new(canvas_id)?;
        Ok(Self { context })
    }

    /// ゲーム世界を描画
    pub fn render(&self, world: &World) -> Result<(), JsValue> {
        // キャンバスをクリア
        self.context.clear()?;
        
        // 背景の描画
        self.render_background()?;
        
        // エンティティの描画
        for entity in world.get_all_entities().iter() {
            // 位置とスプライトの両方を持つエンティティのみ描画
            if let (Some(position), Some(sprite)) = (
                world.get_component::<Position>(*entity),
                world.get_component::<Sprite>(*entity)
            ) {
                self.render_sprite(&self.context.context, position, sprite, None)?;
                
                // カードエンティティの場合は追加情報を描画
                if let Some(card) = world.get_component::<CardInfo>(*entity) {
                    self.render_card_info(&self.context.context, position, card)?;
                }
                
                // ドラッグ中のエンティティに視覚的なフィードバックを追加
                if let Some(draggable) = world.get_component::<Draggable>(*entity) {
                    if draggable.is_dragging {
                        self.render_drag_feedback(&self.context.context, position)?;
                    }
                }
            }
        }
        
        // UI要素の描画（スコア、タイマーなど）
        self.render_ui(world)?;
        
        Ok(())
    }
    
    /// 背景を描画
    fn render_background(&self) -> Result<(), JsValue> {
        let ctx = &self.context.context;
        let width = self.context.width();
        let height = self.context.height();
        
        // 背景色の設定（緑色の背景など）
        ctx.set_fill_style(&JsValue::from_str("#076324"));
        ctx.fill_rect(0.0, 0.0, width, height);
        
        // 背景のパターンや装飾を追加することも可能
        
        Ok(())
    }
    
    /// スプライトを描画
    fn render_sprite(
        &self, 
        ctx: &CanvasRenderingContext2d, 
        position: &Position, 
        sprite: &Sprite,
        _assets: Option<&ResourceManager>
    ) -> Result<(), JsValue> {
        // 画像の代わりに色付きの矩形を描画
        ctx.set_fill_style(&JsValue::from_str(&sprite.color));
        ctx.fill_rect(position.x, position.y, sprite.width, sprite.height);
        
        Ok(())
    }
    
    /// カード情報を描画（数字やスート記号など）
    fn render_card_info(
        &self,
        ctx: &CanvasRenderingContext2d,
        position: &Position,
        card: &CardInfo
    ) -> Result<(), JsValue> {
        // カードが裏向きの場合は情報を表示しない
        if !card.face_up {
            return Ok(());
        }
        
        let x = position.x;
        let y = position.y;
        
        // カードの値とスートを描画
        ctx.set_font("16px Arial");
        ctx.set_fill_style(&JsValue::from_str(if card.is_red() { "#CC0000" } else { "#000000" }));
        
        // カードの値を文字列に変換
        let value_str = card.get_symbol();
        
        // スート記号を取得
        let suit_char = card.get_suit_symbol();
        
        // 左上に値とスートを描画
        ctx.fill_text(&format!("{}{}", value_str, suit_char), x + 5.0, y + 20.0)?;
        
        // 右下にも値とスートを描画（回転して表示）
        ctx.save();
        ctx.translate(x + 70.0, y + 100.0)?;
        ctx.rotate(std::f64::consts::PI)?;
        ctx.fill_text(&format!("{}{}", value_str, suit_char), 0.0, 0.0)?;
        ctx.restore();
        
        Ok(())
    }
    
    /// ドラッグ中の視覚的フィードバックを描画
    fn render_drag_feedback(
        &self,
        ctx: &CanvasRenderingContext2d,
        position: &Position
    ) -> Result<(), JsValue> {
        // ドラッグ中のエンティティに枠線を追加
        ctx.set_stroke_style(&JsValue::from_str("#FFCC00"));
        ctx.set_line_width(2.0);
        ctx.stroke_rect(position.x - 2.0, position.y - 2.0, 74.0, 104.0);
        
        // 発光効果や影などの追加も可能
        
        Ok(())
    }
    
    /// UI要素を描画（スコア、タイマー、ボタンなど）
    fn render_ui(&self, _world: &World) -> Result<(), JsValue> {
        let ctx = &self.context.context;
        let width = self.context.width();
        
        // スコア表示
        ctx.set_font("20px Arial");
        ctx.set_fill_style(&JsValue::from_str("#FFFFFF"));
        ctx.fill_text("スコア: 0", 20.0, 30.0)?;
        
        // タイマー表示
        ctx.fill_text("時間: 00:00", width - 120.0, 30.0)?;
        
        Ok(())
    }
    
    /// レンダーコンテキストの参照を取得
    pub fn context(&self) -> &RenderContext {
        &self.context
    }
} 