// ECS（Entity Component System）モジュール
//
// このモジュールは、ソリティアゲームで使用するECSフレームワークを実装します。
// ECSとは、Entityを中心に、ComponentとSystemを組み合わせるアーキテクチャパターンです。
// - Entity: ゲーム内のオブジェクト（例：カード、カードの山）の基本単位
// - Component: データのみを持つ構造体（例：位置、速度、表示情報）
// - System: ロジックを実装し、Componentを持つEntityを処理する

// サブモジュール
pub mod entity;      // エンティティ管理
pub mod component;   // コンポーネント定義
pub mod system;      // システム定義
pub mod world;       // ワールド（ゲーム全体の状態）
pub mod resources;   // リソース（グローバルな状態）

// モジュール内で使用する型をエクスポート
pub use self::entity::*;
pub use self::component::*;
pub use self::system::*;
pub use self::world::*;
pub use self::resources::*; 