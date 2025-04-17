// network.rs - ネットワーク機能を実装するモジュール
// 将来的にはマルチプレイヤーゲームのために使用される可能性があります

use wasm_bindgen::prelude::*;
use web_sys::{WebSocket, MessageEvent, ErrorEvent, CloseEvent, BinaryType};
use wasm_bindgen::closure::Closure;
use log::{info, error, debug};
use std::cell::RefCell;
use std::rc::Rc;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// ネットワークマネージャークラス
/// WebSocketを使用してサーバーと通信を行う
pub struct NetworkManager {
    ws: Option<WebSocket>,
    url: String,
    connected: bool,
    // コールバック関数
    on_message: Option<Closure<dyn FnMut(MessageEvent)>>,
    on_error: Option<Closure<dyn FnMut(ErrorEvent)>>,
    on_close: Option<Closure<dyn FnMut(CloseEvent)>>,
    on_open: Option<Closure<dyn FnMut(JsValue)>>,
}

impl NetworkManager {
    /// 新しいネットワークマネージャーを作成
    pub fn new(url: &str) -> Self {
        NetworkManager {
            ws: None,
            url: url.to_string(),
            connected: false,
            on_message: None,
            on_error: None,
            on_close: None,
            on_open: None,
        }
    }

    /// WebSocketサーバーに接続
    pub fn connect(&mut self) -> Result<(), JsValue> {
        info!("WebSocketサーバー{}に接続を試みています...", self.url);
        
        let ws = WebSocket::new(&self.url)?;
        
        // バイナリタイプを設定
        ws.set_binary_type(BinaryType::Arraybuffer);
        
        // イベントハンドラーを設定
        let on_open = Closure::wrap(Box::new(move |_| {
            info!("WebSocket接続が確立されました");
        }) as Box<dyn FnMut(JsValue)>);
        ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        self.on_open = Some(on_open);
        
        let on_message = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                let msg = String::from(txt);
                debug!("メッセージを受信しました: {}", msg);
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        self.on_message = Some(on_message);
        
        let on_error = Closure::wrap(Box::new(move |e: ErrorEvent| {
            error!("WebSocketエラー: {:?}", e);
        }) as Box<dyn FnMut(ErrorEvent)>);
        ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        self.on_error = Some(on_error);
        
        let on_close = Closure::wrap(Box::new(move |e: CloseEvent| {
            info!("WebSocket接続が閉じられました。コード: {}, 理由: {}", e.code(), e.reason());
        }) as Box<dyn FnMut(CloseEvent)>);
        ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));
        self.on_close = Some(on_close);
        
        self.ws = Some(ws);
        self.connected = true;
        
        Ok(())
    }
    
    /// メッセージを送信
    pub fn send_message(&self, message: &str) -> Result<(), JsValue> {
        if let Some(ws) = &self.ws {
            if self.connected {
                ws.send_with_str(message)?;
                debug!("メッセージを送信しました: {}", message);
                return Ok(());
            }
        }
        
        error!("WebSocketが接続されていないため、メッセージを送信できません");
        Err(JsValue::from_str("WebSocketが接続されていません"))
    }
    
    /// 接続を閉じる
    pub fn disconnect(&mut self) -> Result<(), JsValue> {
        if let Some(ws) = &self.ws {
            ws.close()?;
            self.connected = false;
            info!("WebSocket接続を閉じました");
        }
        
        Ok(())
    }
    
    /// 接続状態を確認
    pub fn is_connected(&self) -> bool {
        self.connected
    }
    
    /// カスタムメッセージハンドラーを設定
    pub fn set_message_handler(&mut self, handler: Box<dyn FnMut(String)>) -> Result<(), JsValue> {
        if let Some(ws) = &self.ws {
            let handler = Rc::new(RefCell::new(handler));
            
            let handler_clone = handler.clone();
            let on_message = Closure::wrap(Box::new(move |e: MessageEvent| {
                if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                    let msg = String::from(txt);
                    debug!("メッセージを受信しました: {}", msg);
                    let mut handler = handler_clone.borrow_mut();
                    handler(msg);
                }
            }) as Box<dyn FnMut(MessageEvent)>);
            
            ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
            self.on_message = Some(on_message);
            
            return Ok(());
        }
        
        Err(JsValue::from_str("WebSocketが初期化されていません"))
    }
}

impl Drop for NetworkManager {
    fn drop(&mut self) {
        // コールバックをクリア
        self.on_message = None;
        self.on_error = None;
        self.on_close = None;
        self.on_open = None;
        
        // 接続を閉じる
        if let Some(ws) = &self.ws {
            let _ = ws.close();
        }
    }
}

/// ネットワーク機能の初期化
pub fn init() {
    info!("ネットワークモジュールを初期化中...");
    // 実装予定
} 