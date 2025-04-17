use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;
use crate::utils::Vec2;

/// UIテキストを描画
pub fn draw_text(
    context: &CanvasRenderingContext2d,
    text: &str,
    position: Vec2,
    font: &str,
    color: &str,
    align: &str,
    baseline: &str,
) -> Result<(), JsValue> {
    context.save();
    
    context.set_font(font);
    context.set_fill_style(&JsValue::from_str(color));
    context.set_text_align(align);
    context.set_text_baseline(baseline);
    
    context.fill_text(text, position.x, position.y)?;
    
    context.restore();
    
    Ok(())
}

/// ボタンを描画
pub fn draw_button(
    context: &CanvasRenderingContext2d,
    text: &str,
    position: Vec2,
    width: f64,
    height: f64,
    fill_color: &str,
    text_color: &str,
    border_color: &str,
    border_width: f64,
    is_hover: bool,
) -> Result<(), JsValue> {
    context.save();
    
    // ホバー時に色を明るくする
    let fill = if is_hover {
        lighten_color(fill_color, 0.2)
    } else {
        fill_color.to_string()
    };
    
    // 角丸長方形を描画
    draw_rounded_rect(
        context,
        position.x,
        position.y,
        width,
        height,
        5.0,
        &fill,
        border_color,
        border_width,
    )?;
    
    // テキストを描画
    context.set_font("16px Arial");
    context.set_fill_style(&JsValue::from_str(text_color));
    context.set_text_align("center");
    context.set_text_baseline("middle");
    
    context.fill_text(text, position.x + width / 2.0, position.y + height / 2.0)?;
    
    context.restore();
    
    Ok(())
}

/// 角丸の長方形を描画
pub fn draw_rounded_rect(
    context: &CanvasRenderingContext2d,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radius: f64,
    fill_color: &str,
    stroke_color: &str,
    stroke_width: f64,
) -> Result<(), JsValue> {
    context.save();
    
    context.begin_path();
    context.move_to(x + radius, y);
    context.line_to(x + width - radius, y);
    context.arc_to(x + width, y, x + width, y + radius, radius)?;
    context.line_to(x + width, y + height - radius);
    context.arc_to(x + width, y + height, x + width - radius, y + height, radius)?;
    context.line_to(x + radius, y + height);
    context.arc_to(x, y + height, x, y + height - radius, radius)?;
    context.line_to(x, y + radius);
    context.arc_to(x, y, x + radius, y, radius)?;
    context.close_path();
    
    context.set_fill_style(&JsValue::from_str(fill_color));
    context.fill();
    
    context.set_stroke_style(&JsValue::from_str(stroke_color));
    context.set_line_width(stroke_width);
    context.stroke();
    
    context.restore();
    
    Ok(())
}

/// スコアやメッセージなどの情報パネルを描画
pub fn draw_info_panel(
    context: &CanvasRenderingContext2d,
    text: &str,
    position: Vec2,
    width: f64,
    height: f64,
    background_color: &str,
    text_color: &str,
) -> Result<(), JsValue> {
    context.save();
    
    // 背景を描画
    draw_rounded_rect(
        context,
        position.x,
        position.y,
        width,
        height,
        8.0,
        background_color,
        "transparent",
        0.0,
    )?;
    
    // テキストを描画
    context.set_font("14px Arial");
    context.set_fill_style(&JsValue::from_str(text_color));
    context.set_text_align("center");
    context.set_text_baseline("middle");
    
    context.fill_text(text, position.x + width / 2.0, position.y + height / 2.0)?;
    
    context.restore();
    
    Ok(())
}

/// 色を明るくする関数
fn lighten_color(color: &str, amount: f64) -> String {
    // 簡易的なRGB操作（HTMLカラーコードを想定）
    if !color.starts_with('#') || color.len() != 7 {
        return color.to_string();
    }
    
    // #RRGGBBからRGBを抽出
    let r = u8::from_str_radix(&color[1..3], 16).unwrap_or(0);
    let g = u8::from_str_radix(&color[3..5], 16).unwrap_or(0);
    let b = u8::from_str_radix(&color[5..7], 16).unwrap_or(0);
    
    // 明るくする
    let r = ((r as f64) + (255.0 - r as f64) * amount).min(255.0) as u8;
    let g = ((g as f64) + (255.0 - g as f64) * amount).min(255.0) as u8;
    let b = ((b as f64) + (255.0 - b as f64) * amount).min(255.0) as u8;
    
    // 新しい色を生成
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

/// ゲームオーバーやクリア時のモーダルメッセージを描画
pub fn draw_modal(
    context: &CanvasRenderingContext2d,
    text: &str,
    canvas_width: f64,
    canvas_height: f64,
) -> Result<(), JsValue> {
    context.save();
    
    // 半透明の背景
    context.set_fill_style(&JsValue::from_str("rgba(0, 0, 0, 0.7)"));
    context.fill_rect(0.0, 0.0, canvas_width, canvas_height);
    
    // メッセージボックス
    let box_width = 300.0;
    let box_height = 150.0;
    let x = (canvas_width - box_width) / 2.0;
    let y = (canvas_height - box_height) / 2.0;
    
    draw_rounded_rect(
        context,
        x,
        y,
        box_width,
        box_height,
        10.0,
        "#FFFFFF",
        "#000000",
        2.0,
    )?;
    
    // テキスト
    context.set_font("24px Arial");
    context.set_fill_style(&JsValue::from_str("#000000"));
    context.set_text_align("center");
    context.set_text_baseline("middle");
    
    context.fill_text(text, canvas_width / 2.0, canvas_height / 2.0)?;
    
    context.restore();
    
    Ok(())
}

/// プログレスバー（ロード中など）を描画
pub fn draw_progress_bar(
    context: &CanvasRenderingContext2d,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    progress: f64,  // 0.0 ~ 1.0
    background_color: &str,
    fill_color: &str,
    border_color: &str,
) -> Result<(), JsValue> {
    context.save();
    
    // 背景
    draw_rounded_rect(
        context,
        x,
        y,
        width,
        height,
        height / 2.0,
        background_color,
        border_color,
        1.0,
    )?;
    
    // 進捗
    let progress_width = width * progress.max(0.0).min(1.0);
    if progress_width > 0.0 {
        draw_rounded_rect(
            context,
            x,
            y,
            progress_width,
            height,
            height / 2.0,
            fill_color,
            "transparent",
            0.0,
        )?;
    }
    
    context.restore();
    
    Ok(())
} 