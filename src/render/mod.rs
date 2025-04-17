// レンダリングモジュール
//
// このモジュールは、ゲームのレンダリング機能を提供します。
// キャンバス上にカードやUI要素を描画する処理を担当します。

use wasm_bindgen::prelude::*;

/// レンダリングモジュール
/// ゲームの視覚的な表示を担当するコンポーネントとシステムを提供します。
/// CanvasRenderingContext2dを使用してカードやUIエレメントを描画します。

// レンダリングコンポーネントとシステム
pub mod renderer;
pub mod systems;
pub mod card_renderer;
pub mod render_context;

// re-exports
pub use render_context::RenderContext;
// アニメーションシステムは他の場所で定義されている場合があります
// pub use systems::animation::{AnimationManager, AnimationSystem, Animation, AnimationType, EasingType}; 