use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

use crate::ecs::component::{CardInfo, Transform};
use super::RenderContext;

/// 簡易的なTextMetrics実装
/// web_sysのTextMetricsがないため独自に実装
struct SimpleTextMetrics {
    width: f64,
}

impl SimpleTextMetrics {
    fn new(width: f64) -> Self {
        Self { width }
    }
    
    fn width(&self) -> f64 {
        self.width
    }
}

/// カードの描画を担当するレンダラー
pub struct CardRenderer {
    context: RenderContext,
    card_width: f64,
    card_height: f64,
}

impl CardRenderer {
    /// 新しいカードレンダラーを作成
    pub fn new(context: RenderContext) -> Self {
        Self {
            context,
            card_width: 75.0,
            card_height: 105.0,
        }
    }

    /// カードを描画
    pub fn render_card(&self, ctx: &CanvasRenderingContext2d, transform: &Transform, card: &CardInfo) -> Result<(), JsValue> {
        // カードの基本形状を描画
        self.draw_card_shape(ctx, transform.position.x, transform.position.y)?;
        
        // カードが裏向きの場合は裏面を描画
        if !card.face_up {
            self.draw_card_back(ctx, transform.position.x, transform.position.y)?;
            return Ok(());
        }
        
        // カードが表向きの場合は表面を描画
        self.draw_card_face(ctx, transform.position.x, transform.position.y, card)?;
        
        Ok(())
    }
    
    /// カードの基本形状を描画（白背景と枠線）
    fn draw_card_shape(&self, ctx: &CanvasRenderingContext2d, x: f64, y: f64) -> Result<(), JsValue> {
        // カードの白い背景
        ctx.set_fill_style(&JsValue::from_str("#FFFFFF"));
        ctx.fill_rect(x, y, self.card_width, self.card_height);
        
        // カードの枠線
        ctx.set_stroke_style(&JsValue::from_str("#000000"));
        ctx.set_line_width(1.0);
        ctx.stroke_rect(x, y, self.card_width, self.card_height);
        
        Ok(())
    }
    
    /// カードの裏面を描画
    fn draw_card_back(&self, ctx: &CanvasRenderingContext2d, x: f64, y: f64) -> Result<(), JsValue> {
        // 背景パターン（例: 格子模様）
        ctx.set_fill_style(&JsValue::from_str("#0055AA"));
        ctx.fill_rect(x + 3.0, y + 3.0, self.card_width - 6.0, self.card_height - 6.0);
        
        // 中央の装飾パターン
        ctx.set_stroke_style(&JsValue::from_str("#FFFFFF"));
        ctx.set_line_width(1.0);
        
        // 格子模様を描画
        let step = 8.0;
        for i in 0..=(self.card_width as usize / step as usize) {
            let line_x = x + 3.0 + (i as f64 * step);
            ctx.begin_path();
            ctx.move_to(line_x, y + 3.0);
            ctx.line_to(line_x, y + self.card_height - 3.0);
            ctx.stroke();
        }
        
        for i in 0..=(self.card_height as usize / step as usize) {
            let line_y = y + 3.0 + (i as f64 * step);
            ctx.begin_path();
            ctx.move_to(x + 3.0, line_y);
            ctx.line_to(x + self.card_width - 3.0, line_y);
            ctx.stroke();
        }
        
        Ok(())
    }
    
    /// カードの表面を描画（数字とスート）
    fn draw_card_face(&self, ctx: &CanvasRenderingContext2d, x: f64, y: f64, card: &CardInfo) -> Result<(), JsValue> {
        // カードの色を設定（ハートとダイヤは赤、クラブとスペードは黒）
        let color = if card.is_red() { "#CC0000" } else { "#000000" };
        ctx.set_fill_style(&JsValue::from_str(color));
        
        // カードの値を文字列に変換
        let value_str = card.get_symbol();
        
        // スート記号を取得
        let suit_char = card.get_suit_symbol();
        
        // 左上のコーナーに値とスートを描画
        ctx.set_font("18px Arial");
        ctx.fill_text(&value_str, x + 5.0, y + 18.0)?;
        
        ctx.set_font("20px Arial");
        ctx.fill_text(suit_char, x + 5.0, y + 38.0)?;
        
        // 右下のコーナーに値とスートを描画（上下逆に）
        ctx.save();
        ctx.translate(x + self.card_width, y + self.card_height)?;
        ctx.rotate(std::f64::consts::PI)?;
        
        ctx.set_font("18px Arial");
        ctx.fill_text(&value_str, 5.0, 18.0)?;
        
        ctx.set_font("20px Arial");
        ctx.fill_text(suit_char, 5.0, 38.0)?;
        ctx.restore();
        
        // 中央にスートを描画（大きめに）
        self.draw_center_suit(ctx, x, y, card)?;
        
        Ok(())
    }
    
    /// カード中央にスートを描画
    fn draw_center_suit(&self, ctx: &CanvasRenderingContext2d, x: f64, y: f64, card: &CardInfo) -> Result<(), JsValue> {
        let center_x = x + self.card_width / 2.0;
        let center_y = y + self.card_height / 2.0;
        
        // フェイスカード（J, Q, K）の場合は特別な描画も可能
        if card.rank >= 11 {
            self.draw_face_card(ctx, x, y, card)?;
            return Ok(());
        }
        
        // スートの記号を取得
        let suit_char = card.get_suit_symbol();
        
        // 通常のカードの場合、数値に応じてスート記号を複数配置
        ctx.set_font("24px Arial");
        
        match card.rank {
            1 => { // エース
                ctx.set_font("48px Arial");
                let text_metrics = measure_text(ctx, suit_char)?;
                let text_width = text_metrics.width();
                ctx.fill_text(suit_char, center_x - text_width / 2.0, center_y + 12.0)?;
            },
            2..=10 => {
                self.draw_suit_pattern(ctx, x, y, card.rank, suit_char)?;
            },
            _ => {} // フェイスカードは既に処理済み
        }
        
        Ok(())
    }
    
    /// スート記号のパターンを描画（2〜10のカード用）
    fn draw_suit_pattern(&self, ctx: &CanvasRenderingContext2d, x: f64, y: f64, value: u8, suit_char: &str) -> Result<(), JsValue> {
        ctx.set_font("20px Arial");
        let text_metrics = measure_text(ctx, suit_char)?;
        let text_width = text_metrics.width();
        
        // 位置の配列（カードの値によって異なるパターン）
        let positions = match value {
            2 => vec![
                (x + self.card_width / 2.0 - text_width / 2.0, y + 30.0),
                (x + self.card_width / 2.0 - text_width / 2.0, y + self.card_height - 30.0),
            ],
            3 => vec![
                (x + self.card_width / 2.0 - text_width / 2.0, y + 30.0),
                (x + self.card_width / 2.0 - text_width / 2.0, y + self.card_height / 2.0),
                (x + self.card_width / 2.0 - text_width / 2.0, y + self.card_height - 30.0),
            ],
            4 => vec![
                (x + 20.0, y + 30.0),
                (x + self.card_width - 20.0 - text_width, y + 30.0),
                (x + 20.0, y + self.card_height - 30.0),
                (x + self.card_width - 20.0 - text_width, y + self.card_height - 30.0),
            ],
            5 => vec![
                (x + 20.0, y + 30.0),
                (x + self.card_width - 20.0 - text_width, y + 30.0),
                (x + self.card_width / 2.0 - text_width / 2.0, y + self.card_height / 2.0),
                (x + 20.0, y + self.card_height - 30.0),
                (x + self.card_width - 20.0 - text_width, y + self.card_height - 30.0),
            ],
            6 => vec![
                (x + 20.0, y + 30.0),
                (x + self.card_width - 20.0 - text_width, y + 30.0),
                (x + 20.0, y + self.card_height / 2.0),
                (x + self.card_width - 20.0 - text_width, y + self.card_height / 2.0),
                (x + 20.0, y + self.card_height - 30.0),
                (x + self.card_width - 20.0 - text_width, y + self.card_height - 30.0),
            ],
            7 => vec![
                (x + 20.0, y + 30.0),
                (x + self.card_width - 20.0 - text_width, y + 30.0),
                (x + self.card_width / 2.0 - text_width / 2.0, y + 45.0),
                (x + 20.0, y + self.card_height / 2.0),
                (x + self.card_width - 20.0 - text_width, y + self.card_height / 2.0),
                (x + 20.0, y + self.card_height - 30.0),
                (x + self.card_width - 20.0 - text_width, y + self.card_height - 30.0),
            ],
            8 => vec![
                (x + 20.0, y + 30.0),
                (x + self.card_width - 20.0 - text_width, y + 30.0),
                (x + self.card_width / 2.0 - text_width / 2.0, y + 45.0),
                (x + 20.0, y + self.card_height / 2.0),
                (x + self.card_width - 20.0 - text_width, y + self.card_height / 2.0),
                (x + self.card_width / 2.0 - text_width / 2.0, y + self.card_height - 45.0),
                (x + 20.0, y + self.card_height - 30.0),
                (x + self.card_width - 20.0 - text_width, y + self.card_height - 30.0),
            ],
            9 => vec![
                (x + 20.0, y + 25.0),
                (x + self.card_width - 20.0 - text_width, y + 25.0),
                (x + 20.0, y + self.card_height / 3.0),
                (x + self.card_width - 20.0 - text_width, y + self.card_height / 3.0),
                (x + self.card_width / 2.0 - text_width / 2.0, y + self.card_height / 2.0),
                (x + 20.0, y + self.card_height * 2.0 / 3.0),
                (x + self.card_width - 20.0 - text_width, y + self.card_height * 2.0 / 3.0),
                (x + 20.0, y + self.card_height - 25.0),
                (x + self.card_width - 20.0 - text_width, y + self.card_height - 25.0),
            ],
            10 => vec![
                (x + 20.0, y + 25.0),
                (x + self.card_width - 20.0 - text_width, y + 25.0),
                (x + self.card_width / 2.0 - text_width / 2.0, y + 35.0),
                (x + 20.0, y + self.card_height / 3.0),
                (x + self.card_width - 20.0 - text_width, y + self.card_height / 3.0),
                (x + 20.0, y + self.card_height * 2.0 / 3.0),
                (x + self.card_width - 20.0 - text_width, y + self.card_height * 2.0 / 3.0),
                (x + self.card_width / 2.0 - text_width / 2.0, y + self.card_height - 35.0),
                (x + 20.0, y + self.card_height - 25.0),
                (x + self.card_width - 20.0 - text_width, y + self.card_height - 25.0),
            ],
            _ => vec![],
        };
        
        // 全ての位置にスート記号を描画
        for (pos_x, pos_y) in positions {
            ctx.fill_text(suit_char, pos_x, pos_y)?;
        }
        
        Ok(())
    }
    
    /// フェイスカード（J, Q, K）を描画
    fn draw_face_card(&self, ctx: &CanvasRenderingContext2d, x: f64, y: f64, card: &CardInfo) -> Result<(), JsValue> {
        let center_x = x + self.card_width / 2.0;
        let center_y = y + self.card_height / 2.0;
        
        // フェイスカードの文字を取得
        let face_char = match card.rank {
            11 => "J",
            12 => "Q",
            13 => "K",
            _ => "",
        };
        
        // 大きく中央に描画
        ctx.set_font("36px serif");
        let text_metrics = measure_text(ctx, face_char)?;
        let text_width = text_metrics.width();
        ctx.fill_text(face_char, center_x - text_width / 2.0, center_y + 12.0)?;
        
        // スート記号を小さく添える
        let suit_char = card.get_suit_symbol();
        
        ctx.set_font("20px Arial");
        ctx.fill_text(suit_char, center_x - 10.0, center_y + 40.0)?;
        
        Ok(())
    }
    
    /// カードをハイライト表示（選択中や有効なプレイ対象として）
    pub fn highlight_card(&self, ctx: &CanvasRenderingContext2d, transform: &Transform) -> Result<(), JsValue> {
        ctx.set_stroke_style(&JsValue::from_str("#FFCC00"));
        ctx.set_line_width(3.0);
        ctx.stroke_rect(
            transform.position.x - 2.0, 
            transform.position.y - 2.0, 
            self.card_width + 4.0, 
            self.card_height + 4.0
        );
        
        Ok(())
    }
    
    /// カードの移動アニメーションを描画
    pub fn draw_moving_card(&self, ctx: &CanvasRenderingContext2d, transform: &Transform, card: &CardInfo, progress: f64) -> Result<(), JsValue> {
        // 透明度を進行に応じて変化させる
        ctx.set_global_alpha(0.7 + 0.3 * progress);
        
        // カードを描画
        self.render_card(ctx, transform, card)?;
        
        // 設定を元に戻す
        ctx.set_global_alpha(1.0);
        
        Ok(())
    }
    
    /// レンダーコンテキストを取得
    pub fn context(&self) -> &RenderContext {
        &self.context
    }
    
    /// カードの標準サイズを取得
    pub fn card_size(&self) -> (f64, f64) {
        (self.card_width, self.card_height)
    }

    fn draw_card_number(&self, card: &CardInfo, x: f64, y: f64) -> Result<(), JsValue> {
        let ctx = &self.context.context;
        let text = card.get_symbol();
        let color = if card.is_red() {
            "#FF0000"
        } else {
            "#000000"
        };
        
        ctx.set_fill_style(&JsValue::from_str(color));
        ctx.set_font("16px Arial");
        ctx.fill_text(&text, x, y)?;
        
        Ok(())
    }
}

/// テキストの幅を測定
fn measure_text(ctx: &CanvasRenderingContext2d, text: &str) -> Result<SimpleTextMetrics, JsValue> {
    // Web APIのTextMetricsが使えないため、おおよその幅を計算
    // 実際のフォントによって異なるが、簡易的な近似値
    let approx_width = text.len() as f64 * 12.0;
    Ok(SimpleTextMetrics::new(approx_width))
} 