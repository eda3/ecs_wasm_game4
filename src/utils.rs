use wasm_bindgen::prelude::*;

/// パニックが発生したときにコンソールにエラーメッセージを表示するためのフックを設定する
pub fn set_panic_hook() {
    // パニックをキャッチしてJavaScriptコンソールに表示するフックを設定する
    // 開発時にエラーの原因が分かりやすくなるよ👍
    console_error_panic_hook::set_once();
}

/// JavaScriptのconsole.logへのラッパー
#[wasm_bindgen]
pub fn console_log(message: &str) {
    web_sys::console::log_1(&JsValue::from_str(message));
}

/// JavaScriptのconsole.errorへのラッパー
#[wasm_bindgen]
pub fn console_error(message: &str) {
    web_sys::console::error_1(&JsValue::from_str(message));
}

/// 現在の時間をミリ秒で取得
pub fn get_current_time() -> Result<f64, JsValue> {
    // ブラウザのパフォーマンスAPIを使って現在時刻を取得
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("ウィンドウが見つかりません"))?;
    let performance = window.performance().ok_or_else(|| JsValue::from_str("パフォーマンスAPIが利用できません"))?;
    Ok(performance.now())
}

/// 二つの値の間の距離を計算
pub fn distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
}

/// 指定した範囲内に値をクランプ（制限）する
pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// 二次元ベクトルを表す補助構造体
#[derive(Clone, Copy, Debug, Default)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    /// 新しいVec2を作成
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    
    /// ゼロベクトル
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
    
    /// ベクトルの長さ（大きさ）を計算
    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    
    /// 正規化されたベクトル（長さが1のベクトル）を返す
    pub fn normalize(&self) -> Self {
        let length = self.length();
        if length > 0.0 {
            Self {
                x: self.x / length,
                y: self.y / length,
            }
        } else {
            *self
        }
    }
    
    /// 別のベクトルとの距離を計算
    pub fn distance(&self, other: &Self) -> f64 {
        ((other.x - self.x).powi(2) + (other.y - self.y).powi(2)).sqrt()
    }
    
    /// 別のベクトルとの内積を計算
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y
    }
    
    /// スカラー値を掛け算
    pub fn scale(&self, scalar: f64) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
    
    /// 別のベクトルを足す
    pub fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
    
    /// 別のベクトルを引く
    pub fn subtract(&self, other: &Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
} 