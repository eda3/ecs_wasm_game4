use wasm_bindgen::JsValue;
use wasm_bindgen::JsCast;
use web_sys::CanvasRenderingContext2d;

/// レンダリングコンテキスト
/// キャンバス要素とその2Dレンダリングコンテキストを管理します
pub struct RenderContext {
    pub canvas: web_sys::HtmlCanvasElement,
    pub context: web_sys::CanvasRenderingContext2d,
    pub dpi_scale: f64,
}

impl RenderContext {
    /// 新しいレンダリングコンテキストを作成
    pub fn new(canvas_id: &str) -> Result<Self, JsValue> {
        // ドキュメントからキャンバス要素を取得
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or_else(|| JsValue::from_str(&format!("キャンバス要素 '{}' が見つかりません", canvas_id)))?
            .dyn_into::<web_sys::HtmlCanvasElement>()?;
        
        // 2Dレンダリングコンテキストを取得
        let context = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("2Dコンテキストを取得できませんでした"))?
            .dyn_into::<web_sys::CanvasRenderingContext2d>()?;
        
        // デバイスのピクセル比を取得（高DPIディスプレイ対応）
        let window = web_sys::window().unwrap();
        let dpi_scale = window.device_pixel_ratio();
        
        // キャンバスのサイズをDPIに合わせて調整
        let width = canvas.width() as f64;
        let height = canvas.height() as f64;
        
        canvas.set_width((width * dpi_scale) as u32);
        canvas.set_height((height * dpi_scale) as u32);
        
        // CSSのサイズを維持
        let style = canvas.style();
        style.set_property("width", &format!("{}px", width))?;
        style.set_property("height", &format!("{}px", height))?;
        
        // スケーリングを適用
        context.scale(dpi_scale, dpi_scale)?;
        
        Ok(Self {
            canvas,
            context,
            dpi_scale,
        })
    }
    
    /// キャンバスをクリア
    pub fn clear(&self) -> Result<(), JsValue> {
        let width = self.canvas.width() as f64 / self.dpi_scale;
        let height = self.canvas.height() as f64 / self.dpi_scale;
        
        self.context.clear_rect(0.0, 0.0, width, height);
        Ok(())
    }
    
    /// キャンバスの幅を取得
    pub fn width(&self) -> f64 {
        self.canvas.width() as f64 / self.dpi_scale
    }
    
    /// キャンバスの高さを取得
    pub fn height(&self) -> f64 {
        self.canvas.height() as f64 / self.dpi_scale
    }
    
    /// レンダリングコンテキストを取得
    pub fn context(&self) -> &web_sys::CanvasRenderingContext2d {
        &self.context
    }
} 