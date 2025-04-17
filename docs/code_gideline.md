# ECS WebAssembly Game コード品質ガイドライン

## 1. コードスタイル

### 1.1 命名規則
- **構造体**: PascalCase
  ```rust
  struct PlayerEntity;
  struct GameState;
  ```
- **トレイト**: PascalCase
  ```rust
  trait Component;
  trait System;
  ```
- **関数**: snake_case
  ```rust
  fn create_entity();
  fn update_game_state();
  ```
- **変数**: snake_case
  ```rust
  let entity_id;
  let game_state;
  ```
- **定数**: SCREAMING_SNAKE_CASE
  ```rust
  const MAX_ENTITIES: usize = 1000;
  const TARGET_FPS: u32 = 60;
  ```

### 1.2 フォーマット
- インデント: 4スペース
- 行の長さ: 最大100文字
- トレイト境界: 複数行で記述
  ```rust
  impl<T> System for MySystem<T>
  where
      T: Component + Send + Sync,
  {
      // ...
  }
  ```

### 1.3 ドキュメント
- モジュールレベル: `//!` を使用
  ```rust
  //! ゲームエンティティモジュール
  //! 
  //! ゲーム固有のエンティティとコンポーネントを実装します。
  ```
- 構造体/トレイト: `///` を使用
  ```rust
  /// プレイヤーエンティティ
  /// 
  /// プレイヤーキャラクターを表すエンティティです。
  pub struct Player;
  ```
- 関数: `///` を使用
  ```rust
  /// 新しいプレイヤーエンティティを作成します。
  /// 
  /// # 引数
  /// 
  /// * `world` - エンティティを作成するワールド
  /// 
  /// # 戻り値
  /// 
  /// 作成されたエンティティ、またはエラー
  pub fn create(world: &mut World) -> Result<Entity, JsValue>;
  ```

### 1.4 グローバル変数とスレッドセーフティ
- **lazy_staticの使用を避ける**理由
  - 依存クレートが増える
  - 状態の初期化に関する制御が制限される
  - キャッシュのような場面では`OnceLock`や`thread_local`の方が適切
  
- **代替方法**
  - `std::sync::OnceLock`（Rustの標準ライブラリ）を使用してスレッドセーフなグローバル変数を実装
    ```rust
    use std::sync::OnceLock;
    
    static COUNTER: OnceLock<Mutex<i32>> = OnceLock::new();
    
    fn get_counter() -> &'static Mutex<i32> {
        COUNTER.get_or_init(|| Mutex::new(0))
    }
    ```
  
  - `thread_local!`マクロを使用してスレッドローカルストレージを実装（WASMのシングルスレッド環境で特に有用）
    ```rust
    use std::cell::RefCell;
    
    thread_local! {
        static THREAD_COUNTER: RefCell<i32> = RefCell::new(0);
    }
    
    // 使用例
    THREAD_COUNTER.with(|counter| {
        *counter.borrow_mut() += 1;
        println!("Counter: {}", *counter.borrow());
    });
    ```

  - WASMの場合、`thread_local!`の方が適切なケースが多い
    - Webブラウザ環境はシングルスレッドで動作するため
    - `Rc`や`RefCell`を利用できる（`Send`/`Sync`トレイトが必要ない）
    - グローバル変数へのアクセスが簡潔になる

- **グローバルなゲームインスタンスの管理**
  - WebAssemblyからアクセス可能なグローバルインスタンスの実装
    ```rust
    thread_local! {
        static GAME_INSTANCE: RefCell<Option<Rc<RefCell<GameInstance>>>> = RefCell::new(None);
    }
    
    // JavaScript向けのエクスポート関数で、グローバルインスタンスを利用
    #[wasm_bindgen]
    pub fn update_mouse_position(x: f32, y: f32) -> Result<(), JsValue> {
        // グローバルインスタンスを安全に利用
        GAME_INSTANCE.with(|instance| {
            if let Some(instance_rc) = &*instance.borrow() {
                let mut game = instance_rc.borrow_mut();
                // 処理...
                Ok(())
            } else {
                Err(JsValue::from_str("ゲームが初期化されていません"))
            }
        })
    }
    ```

### 1.5 未使用変数の処理
- **アンダースコアプレフィックスの使用**
  - 意図的に使用しない変数には、名前の先頭にアンダースコアを付ける
    ```rust
    // 良い例: 未使用変数にアンダースコアプレフィックスを付ける
    fn update_state(&mut self, _delta_time: f32) {
        // delta_timeを使わずに処理
    }
    
    // 避けるべき例: 未使用変数をそのままにする
    fn update_state(&mut self, delta_time: f32) {
        // delta_timeを使わずに処理 → 警告が出る
    }
    ```
  
  - 関数シグネチャの一貫性を保ちつつ警告を抑制できる
    - 将来的に変数を使う可能性がある場合に有用
    - インターフェースの一貫性が重要な場合に特に便利
  
  - 完全に無視する場合は単なるアンダースコアを使用
    ```rust
    // 値を完全に無視する場合
    for _ in 0..10 {
        // インデックスを使わないループ
        perform_action();
    }
    ```
  
  - クロージャでの未使用パラメータの例
    ```rust
    // 問題のあるコード: 未使用の引数にアンダースコアが付いていない
    entity_set.into_iter().map(|id| {
        // idを使わずにエンティティを作成
        let entity = Entity::new();
        entity
    });
    
    // 修正後: アンダースコアプレフィックスを付けて意図を明確に
    entity_set.into_iter().map(|_id| {
        // _idは意図的に使用しないことを示す
        let entity = Entity::new();
        entity
    });
    ```
  
  - システムの実装での例
    ```rust
    // InputSystemの実装例
    impl System for InputSystem {
        fn name(&self) -> &'static str {
            "InputSystem"
        }
        
        fn phase(&self) -> SystemPhase {
            SystemPhase::Input
        }
        
        fn priority(&self) -> SystemPriority {
            SystemPriority::new(0) // 入力処理は優先度0（最優先）
        }

        // 良い例: 未使用パラメータにアンダースコアプレフィックスを付ける
        fn run(&mut self, world: &mut World, _resources: &mut ResourceManager, _delta_time: f32) -> Result<(), JsValue> {
            // resourcesやdelta_timeを使わずに処理
            // ...
            Ok(())
        }
    }
    ```
  
  - 同様にNetworkSyncSystemでの例
    ```rust
    // NetworkSyncSystemの実装例
    impl System for NetworkSyncSystem {
        fn name(&self) -> &'static str {
            "NetworkSyncSystem"
        }
        
        fn phase(&self) -> SystemPhase {
            SystemPhase::NetworkSync
        }
        
        fn priority(&self) -> SystemPriority {
            SystemPriority::new(0)
        }

        // 良い例: 未使用パラメータにアンダースコアプレフィックスを付ける
        fn run(&mut self, world: &mut World, _resources: &mut ResourceManager, _delta_time: f32) -> Result<(), JsValue> {
            // resourcesとdelta_timeを使わない場合、アンダースコアプレフィックスを付ける
            let now = Date::now();
            // ... その他の処理 ...
            Ok(())
        }
    }
    ```
  
  - 同様にNetworkCompressionSystemでの例
    ```rust
    // NetworkCompressionSystemの実装例
    impl System for NetworkCompressionSystem {
        fn name(&self) -> &'static str {
            "NetworkCompressionSystem"
        }

        fn run(&mut self, _world: &mut World, resources: &mut ResourceManager, _delta_time: f32) -> Result<(), JsValue> {
            // 現在の時間を取得
            let _current_time = js_sys::Date::now();
            
            // worldとdelta_timeを使わない場合、アンダースコアプレフィックスを付ける
            // 一時変数も同様にアンダースコアプレフィックスを付ける
            
            // 処理すべきエンティティがあればここで圧縮処理を実行
            // 実際の実装では、このシステムは他のネットワークシステムと連携して動作します
            
            // 性能ログ出力（デバッグ用）
            if let Some(mode) = resources.get::<DebugMode>() {
                if mode.show_debug_info {
                    println!("NetworkCompressionSystem: 現在のモード={:?}, 帯域={:.1}KB/s", 
                        self.adaptive_mode,
                        self.bandwidth_usage.calculate_current_bandwidth() / 1000.0);
                }
            }
            
            Ok(())
        }
        
        fn phase(&self) -> SystemPhase {
            SystemPhase::Update
        }
        
        fn priority(&self) -> SystemPriority {
            SystemPriority::new(0) // 標準優先度
        }
    }
    ```
  
  - イベントハンドラでの使用例
    ```rust
    // GameStateクラスのマウス入力処理
    // 未使用パラメータにアンダースコアを付けて警告を抑制
    fn handle_splash_mouse(&mut self, _x: f32, _y: f32, _button: u8) -> Result<(), JsValue> {
        // クリック位置やボタン種類に関わらず同じ処理をする場合
        web_sys::console::log_1(&"スプラッシュ画面をクリック: メインメニューへ遷移します".into());
        self.current_state = GameStateType::MainMenu;
        Ok(())
    }
    
    // 一部のパラメータだけを使用する場合も同様
    fn handle_playing_mouse(&mut self, _x: f32, _y: f32, button: u8) -> Result<(), JsValue> {
        // ボタン種類だけをチェックする場合
        if button == 2 {
            web_sys::console::log_1(&"右クリック: ゲームをポーズします".into());
            self.current_state = GameStateType::Paused;
        }
        Ok(())
    }
    ```
    
  - 一時変数の未使用警告を抑制する例
    ```rust
    // 変数を宣言したが使用しない場合
    let _canvas_height = self.canvas.height() as f32;
    // 警告が出ないようにアンダースコアプレフィックスを付ける
    ```
  
  - クロージャでの使用例
    ```rust
    // 未使用の引数がある場合
    buttons.iter().for_each(|button| {
        render_button(button);
    });
    
    // event引数を使わない場合
    canvas.set_onclick(Some(move |_event| {
        handle_click();
    }));
    ```

- **未使用Result値の処理**
  - `Result`型を返す関数の戻り値は必ず処理する
    ```rust
    // 避けるべき例: Result値を無視する
    self.context.translate(x, y); // 警告: unused `Result` that must be used
    
    // 良い例: アンダースコア変数を使用して意図的に無視することを明示
    let _ = self.context.translate(x, y);
    ```
  
  - WebCanvas APIなどの外部APIを使用する場合に特に重要
    ```rust
    // Rendererクラスでの正しいCanvas API呼び出し例
    fn draw_sprite(&mut self, sprite: &Sprite) -> Result<(), JsValue> {
        // 変換前の状態を保存
        let _ = self.context.save();
        
        // 位置設定
        let screen_x = sprite.x;
        let screen_y = sprite.y;
        let _ = self.context.translate(screen_x, screen_y);
        
        // 回転設定（回転がある場合のみ）
        if sprite.rotation != 0.0 {
            let _ = self.context.rotate(sprite.rotation);
        }
        
        // スケール設定（デフォルトではない場合のみ）
        if sprite.scale_x != 1.0 || sprite.scale_y != 1.0 {
            let _ = self.context.scale(sprite.scale_x, sprite.scale_y);
        }
        
        // 描画処理...
        
        // 状態を復元
        let _ = self.context.restore();
        
        Ok(())
    }
    ```
  
  - web_sys APIでも同様に適用
    ```rust
    // 避けるべき例: console.log_1のResult値を無視する
    web_sys::console::log_1(&"メッセージ".into()); // 警告: unused `Result` that must be used
    
    // 良い例: アンダースコア変数を使用して意図的に無視することを明示
    let _ = web_sys::console::log_1(&"メッセージ".into());
    
    // デバッグログを出力する関数での例
    fn log_entity_sync(&self, entity: Entity, bytes: usize) {
        if self.config.debug_mode {
            let _ = web_sys::console::log_1(&format!("エンティティ {:?} を同期: {}バイト", entity, bytes).into());
        }
    }
    ```
  
  - インターフェースの実装においても、使わないパラメータにはアンダースコアプレフィックスを付ける
    ```rust
    // トレイトの実装における未使用パラメータの例
    pub trait MessageCompressor: Send + Sync {
        /// スナップショットを圧縮
        fn compress(&self, snapshot: &LocalEntitySnapshot) -> LocalEntitySnapshot;
        
        /// 圧縮効率を推定（0.0〜1.0、値が小さいほど効率が良い）
        fn estimate_efficiency(&self, snapshot: &LocalEntitySnapshot) -> f32;
    }

    impl MessageCompressor for DefaultMessageCompressor {
        fn compress(&self, snapshot: &LocalEntitySnapshot) -> LocalEntitySnapshot {
            // snapshotパラメータを使用している
            let compressed = snapshot.clone();
            // 圧縮処理...
            compressed
        }
        
        // 良い例: 使用しないパラメータにアンダースコアを付ける
        fn estimate_efficiency(&self, _snapshot: &LocalEntitySnapshot) -> f32 {
            // snapshotパラメータを使わないが、インターフェースの一貫性のために存在している
            // 簡易実装なので固定値を返す
            0.5
        }
    }
    ```
  
  - エラーを意図的に無視することを明示的に表現することで:
    - コンパイラ警告が出なくなる
    - コードの意図が明確になる
    - エラーハンドリングの方針を一貫させられる

### 1.5 未使用の変数

未使用の変数がある場合、Rustコンパイラは警告を出します。これは、コードの不具合や無駄を示す重要なシグナルです。未使用変数を放置すると、コードの品質が低下し、可読性や保守性にも悪影響を与えます。

未使用の変数を持つ関数やパラメータがある場合は、次のいずれかの対応を取ってください：

1. **変数に使用目的がある場合**：その変数の使用方法を実装してください。
2. **変数が現在不要だが、将来的に使用する可能性がある場合**：その変数名の前にアンダースコア（`_`）をつけてください。これにより、Rustコンパイラに「この変数は意図的に使用していない」と伝えることができます。
3. **変数が不要であることが確定している場合**：その変数を削除してください。関数シグネチャから不要なパラメータを取り除くことで、コードがよりクリーンになります。

#### 例：

👎 **悪い例**
```rust
fn process_input(delta_time: f32, resources: &Resources) -> bool {
    // delta_timeとresourcesを使わないコード
    true
}
```

👍 **良い例**
```rust
fn process_input(_delta_time: f32, _resources: &Resources) -> bool {
    // delta_timeとresourcesは現在使われていないが、将来的に必要になる可能性があるためアンダースコア接頭辞を使用
    true
}
```

👍 **より良い例（不要なパラメータを削除）**
```rust
fn process_input() -> bool {
    // 不要なパラメータを完全に削除
    true
}
```

#### 実際の実装例：

`InputSystem`の`run`メソッドでは、ECSのシステムインターフェースに準拠するため、使用していないパラメータに対してアンダースコア接頭辞を使用しています：

```rust
impl System for InputSystem {
    fn run(&mut self, _delta_time: f32, _resources: &Resources) {
        // delta_timeとresourcesは現在使われていないが、システムインターフェースに必要
        // 適切にアンダースコア接頭辞を付けて、コンパイラ警告を抑制
        
        // 入力処理のコード...
    }
}
```

クロージャでの未使用引数も同様に処理できます：

```rust
let handler = |_event: Event, state: &mut GameState| {
    // eventは使われていないのでアンダースコア接頭辞を付ける
    state.update();
};
```

### 1.6 未使用のResult値の処理
- **戻り値としてのResultを無視しない**
  - エラーハンドリングはRustの重要な機能であり、`Result`型の戻り値を無視するとエラーが検出されなくなります
    ```rust
    // 問題のあるコード: Result値が無視されている
    self.context.translate(x, y);
    self.context.rotate(angle);
    
    // 良い例: let _ = を使って意図的に無視していることを示す
    let _ = self.context.translate(x, y);
    let _ = self.context.rotate(angle);
    ```
  
  - エラー処理が必要ない場合でも、`let _ =`構文を使って意図的に無視していることを明示する
    - コンパイラの警告を抑制
    - コードの意図を明確に伝える
    - 将来的なコードレビューや保守の際に混乱を防ぐ
  
  - Web Canvas API操作での例
    ```rust
    // 問題のあるコード: Canvas APIの操作結果が無視されている
    fn draw_sprite(&mut self, sprite: &Sprite) {
        self.context.translate(sprite.x, sprite.y);
        self.context.rotate(sprite.rotation);
        self.context.scale(sprite.scale_x, sprite.scale_y);
        // 他の描画処理...
    }
    
    // 修正後: Result値を明示的に処理
    fn draw_sprite(&mut self, sprite: &Sprite) {
        let _ = self.context.translate(sprite.x, sprite.y);
        let _ = self.context.rotate(sprite.rotation);
        let _ = self.context.scale(sprite.scale_x, sprite.scale_y);
        // 他の描画処理...
    }
    ```
  
  - エラー伝播が必要な場合は`?`演算子を使用
    ```rust
    // エラーを上位に伝播させる場合
    fn draw_complex_sprite(&mut self, sprite: &Sprite) -> Result<(), JsValue> {
        self.context.translate(sprite.x, sprite.y)?;
        self.context.rotate(sprite.rotation)?;
        self.context.scale(sprite.scale_x, sprite.scale_y)?;
        // 他の描画処理...
        Ok(())
    }
    ```

- **エラーの意図的な無視と明示的な処理**
  - エラーを無視する理由が明確な場合は、コメントで説明する
    ```rust
    // アニメーションフレームの描画失敗は許容するため意図的に無視
    let _ = self.context.draw_image_with_html_image_element(
        &sprite.image,
        sprite.src_x, 
        sprite.src_y,
        sprite.src_width, 
        sprite.src_height,
        0.0, 
        0.0,
        sprite.width, 
        sprite.height,
    );
    ```
  
  - 複数の操作が連続する場合でも、各操作の結果を個別に処理する
    ```rust
    // 各操作の結果を個別に処理
    let _ = self.context.save();
    let _ = self.context.set_global_alpha(sprite.opacity);
    // 描画処理...
    let _ = self.context.restore();
    ```

## 2. エラーハンドリング

### 2.1 エラー型
- カスタムエラー型の定義
  ```rust
  #[derive(Debug)]
  pub struct GameError {
      message: String,
      source: Option<Box<dyn std::error::Error>>,
  }
  ```

### 2.2 エラー処理
- `Result`型の使用
  ```rust
  pub fn load_resource(path: &str) -> Result<Resource, GameError>;
  ```
- エラーの伝播
  ```rust
  fn process_input() -> Result<(), GameError> {
      let input = read_input()?;
      validate_input(&input)?;
      Ok(())
  }
  ```

### 2.3 Option型の扱い
- `unwrap_or`を使用したデフォルト値の設定
  ```rust
  // Option<f32>をf32に安全に変換
  position.z.unwrap_or(0.0);
  ```
- パターンマッチングを使用した安全な処理
  ```rust
  match optional_value {
      Some(value) => process_value(value),
      None => handle_missing_value(),
  }
  ```
- メソッド呼び出し時のOption型の適切な処理
  ```rust
  // 良い例: Optionの可能性を考慮したコード
  snapshot.with_position([position.x, position.y, position.z.unwrap_or(0.0)]);
  
  // 避けるべき例: 不用意なunwrap
  snapshot.with_position([position.x, position.y, position.z.unwrap()]); // パニックの危険性
  ```
- 型の一貫性の確保
  ```rust
  // 構造体の定義で明示的にOptionであることを宣言
  pub struct PositionComponent {
      pub x: f32,
      pub y: f32,
      pub z: Option<f32>, // 必須でない値はOptionで表現
  }
  ```

### 2.4 型変換のベストプラクティス
- 明示的な型変換を使用
  ```rust
  // u8からu32への安全な変換
  let key_code: u32 = mouse_button.into();
  
  // 数値型の変換
  let float_value: f32 = integer_value as f32;
  ```
- TryIntoトレイトを使用した安全な変換
  ```rust
  // 失敗する可能性のある変換（例: u64からu32への変換）
  let smaller_value: u32 = larger_value.try_into().unwrap_or(0);
  ```
- メソッド呼び出し時の型変換
  ```rust
  // 良い例: 型変換を明示的に行う
  .bind_key("attack", MOUSE_LEFT.into());  // u8からu32へ
  
  // 避けるべき例: 暗黙的な型変換に頼る
  .bind_key("attack", MOUSE_LEFT);  // 型が一致しない
  ```
- 文字列からの変換
  ```rust
  // 文字列からの数値変換
  let value = str_value.parse::<i32>().unwrap_or(0);
  ```
- 異なるコレクション型間の変換
  ```rust
  // Vec<T> から HashMap<K, V> への変換例
  let mut map = HashMap::new();
  for item in vec {
      map.insert(generate_key(&item), item);
  }
  
  // 避けるべき例: 互換性のないコレクション型を直接渡す
  some_function(vec); // HashMap<K, V>を期待する関数にVec<T>を渡す
  ```

- 列挙型の種類を判定して適切な処理を行う
  ```rust
  // 良い例: matchを使った列挙型の種類に基づく文字列生成
  let type_name = match data {
      DataType::Integer(_) => "integer",
      DataType::Float(_) => "float",
      DataType::String(_) => "string",
      DataType::Boolean(_) => "boolean",
  };
  
  // 避けるべき例: 存在しないメソッドを呼び出す
  let type_name = data.get_type_name(); // そのようなメソッドが実装されていない
  ```

### 2.5 所有権と借用のパターン
- 参照の衝突を避ける
  ```rust
  // 問題のあるコード: 同じオブジェクトを不変参照と可変参照で同時に使用
  if let Some(component) = world.get_component::<Component>(entity) {
      for item in &component.items {
          // エラー: worldはすでに不変参照されている
          handler(entity, world, item)?;
      }
  }
  
  // 改善案1: 必要なデータを事前に収集
  let actions_to_execute = entities
      .iter()
      .filter_map(|entity| {
          world.get_component::<Component>(*entity).map(|component| 
              (entity, component.get_actions())
          )
      })
      .collect::<Vec<_>>();
  
  // 収集後に処理を実行（所有権の衝突なし）
  for (entity, actions) in actions_to_execute {
      for action in actions {
          handler(*entity, world, action)?;
      }
  }
  ```

- クロージャと所有権
  ```rust
  // 良い例: moveキーワードで所有権を明示的に移動
  let processor = move |data| {
      // dataの所有権を取得
      process_owned_data(data);
  };
  
  // 良い例: 参照のみを使用
  let reader = |data: &Data| {
      // dataの参照のみを使用
      read_data(data);
  };
  ```

- RefCellによる内部可変性
  ```rust
  use std::cell::RefCell;
  
  // 不変参照しか持てない状況で可変性が必要な場合
  struct Component {
      data: RefCell<Vec<String>>,
  }
  
  impl Component {
      fn add_item(&self, item: String) {
          // 不変参照を持ちながらも内部データを変更可能
          self.data.borrow_mut().push(item);
      }
  }
  ```

- 自己参照構造体での所有権問題
  ```rust
  // 問題のあるコード: selfの一部を保持したまま別のメソッドを呼び出す
  fn process(&mut self) {
      let state = &mut self.state;
      // エラー: stateを借用している間にselfの別のメソッドを呼び出せない
      let data = self.compute_data();
      state.update(data);
  }
  
  // 改善案: 必要なデータを先に計算して、その後に状態を更新
  fn process(&mut self) {
      // 先にデータを計算
      let data = self.compute_data();
      // 後から状態を更新
      self.state.update(data);
  }
  ```

- 所有権移動を避けるための一時変数の活用
  ```rust
  // 問題のあるコード: with_positionメソッドが所有権を消費する場合
  snapshot.with_position([x, y, z]); // snapshotの所有権が移動する
  snapshot.with_velocity([vx, vy, vz]); // エラー: 既に移動したsnapshotを使用

  // 改善策1: 直接フィールドに代入
  let pos = [x, y, z];
  snapshot.position = Some(pos);
  snapshot.velocity = Some([vx, vy, vz]);

  // 改善策2: メソッドが&mut selfを取るように設計
  snapshot.set_position([x, y, z]); // 所有権を移動しない
  snapshot.set_velocity([vx, vy, vz]); // OK
  ```

### 2.5.1 所有権と借用の追加パターン

- HashMapへのアクセス中に自己メソッドを呼び出すパターン
  ```rust
  // 問題のあるコード: HashMapの可変借用中に自己メソッドを呼び出す
  fn process_entity(&mut self, entity: Entity) {
      let state = self.entity_states.get_mut(&entity).unwrap();
      
      // エラー: 既にself.entity_statesを可変借用中に別のselfメソッドを呼び出している
      let data = self.calculate_data(entity);
      state.update(data);
  }
  
  // 改善策: 必要なデータを先に計算してから、可変借用を行う
  fn process_entity(&mut self, entity: Entity) {
      // 1. 先に必要なデータを計算または取得
      let data = self.calculate_data(entity);
      let hash_values = self.compute_hashes(entity);
      
      // 2. その後でHashMapに対する操作を行う
      let state = self.entity_states.get_mut(&entity).unwrap();
      state.update(data, hash_values);
  }
  ```

- クローン活用による借用問題回避パターン
  ```rust
  // 問題のあるコード: 可変借用中に同じ構造体の一部を再度使用
  fn update(&mut self) {
      let element = self.elements.get_mut(0).unwrap();
      
      // エラー: self.elementsが既に可変借用されているのにアクセスしている
      for other in &self.elements[1..] {
          element.combine(other);
      }
  }
  
  // 改善策: 処理対象のデータを一時的にクローンして借用を終了させる
  fn update(&mut self) {
      // 処理対象の要素をクローンして取り出し、借用を終了させる
      let mut element = self.elements[0].clone();
      
      // これで借用の競合なしに処理できる
      for other in &self.elements[1..] {
          element.combine(other);
      }
      
      // 結果を書き戻す
      self.elements[0] = element;
  }
  ```

- バッファリングによる複数回の計算回避パターン
  ```rust
  // 問題のあるコード: 同じ計算を複数回実行している
  for component in &snapshot.components {
      // 計算を毎回実行している
      let hash = self.compute_component_hash(component);
      let name = self.get_component_name(component);
      
      // hashとnameを使用
      // ...
  }
  
  // 改善策: 計算を一度だけ実行してキャッシュする
  // 1. 事前に必要な情報を収集
  let mut component_data = Vec::new();
  for component in &snapshot.components {
      let hash = self.compute_component_hash(component);
      let name = self.get_component_name(component);
      component_data.push((component, hash, name));
  }
  
  // 2. 収集した情報を使って処理
  for (component, hash, name) in component_data {
      // ...
  }
  ```

これらのパターンを適用することで、Rustの所有権システムに起因する多くの問題を効果的に解決できます。重要なのは、データの生存期間を考慮し、競合する借用が発生しないように設計することです。

### 2.5.2 Option型と通常型の区別

Rustでは特にOption型と通常の型を区別する際に型の不一致エラーが発生することがあります。以下のパターンに注意しましょう。

- 通常型とOption型の混同
  ```rust
  // 問題のあるコード: input.movementが(f32, f32)型なのに、Option型として扱っている
  if let Some((move_x, move_y)) = input.movement {
      // 処理...
  }
  
  // 正しいコード: 型に合わせて適切なパターンマッチングを使用
  let (move_x, move_y) = input.movement;
  ```

- Option型のパターンマッチング
  ```rust
  // 良い例: Option<T>型には`Some`パターンを使用
  if let Some(value) = optional_value {
      // valueはT型
  }
  
  // 良い例: Option<(T, U)>型のタプルでも同様
  if let Some((first, second)) = optional_tuple {
      // firstはT型、secondはU型
  }
  ```

- 関数内での一貫した型の使用
  ```rust
  // 問題のあるコード: 引数がOption<(f32, f32)>なのに処理が不一致
  fn process_input(input: Option<(f32, f32)>) {
      let (x, y) = input;  // エラー: パターンの型が不一致
  }
  
  // 正しいコード: Option型は適切に処理
  fn process_input(input: Option<(f32, f32)>) {
      if let Some((x, y)) = input {
          // 処理...
      }
  }
  ```

- 値の設定時の型一貫性
  ```rust
  // 問題のあるコード: movement は Option<(f32, f32)> 型だが、タプルを直接代入
  predicted_input.movement = (px, py);  // エラー: 型が不一致
  
  // 正しいコード: Option型に合わせて値をラップ
  predicted_input.movement = Some((px, py));
  ```

- 中間値を使った型の明確化
  ```rust
  // 良い例: 変数宣言で型を明確にする
  let movement: Option<(f32, f32)> = Some((1.0, 2.0));
  
  // 型の不整合を防ぐためのチェック
  // コンパイル時に型エラーを検出
  fn _type_check<T>(_: &Option<T>, _: &T) {}
  _type_check(&movement, &(1.0, 2.0));  // OK: movementはOption<(f32, f32)>型
  ```

Option型と通常の型の区別は、Rustの型システムにおいて特に注意が必要です。適切なパターンマッチングと型アノテーションを使用することで、これらの問題を回避できます。

### 2.5.3 明示的な型アノテーションの活用

Rustでは型推論が強力ですが、特定の状況では明示的な型アノテーションが必要になります。適切な型アノテーションを使用することでコードの意図が明確になり、コンパイルエラーを防ぐことができます。

- 数値の型変換と数学的操作
  ```rust
  // 問題のあるコード: 型推論が曖昧な場合
  let difference = (value1 - value2).abs(); // どの型のabsメソッドを使うべきか不明確
  
  // 修正例: 型キャストを明示的に行ってから操作を適用
  let difference = ((value1 - value2) as f64).abs(); // 型が明確
  ```

- 複雑な式の型アノテーション
  ```rust
  // 問題のあるコード: 複雑な計算式で型が曖昧
  let result = values.iter().map(|x| (x * factor).sin()).collect(); // 戻り値の型が不明確
  
  // 修正例: 戻り値の型を明示
  let result: Vec<f64> = values.iter().map(|x| (x * factor).sin()).collect(); // 型が明確
  ```

- メソッドチェーンでの型アノテーション
  ```rust
  // 問題のあるコード: メソッドチェーンのどこかで型が曖昧になっている
  let processed = input.process().transform().calculate(); // エラー
  
  // 修正例: 中間変数で型を明示
  let processed: ProcessedData = input.process();
  let transformed: TransformedData = processed.transform();
  let result = transformed.calculate();
  ```

- ジェネリック関数での型パラメータの明示
  ```rust
  // 問題のあるコード: 型推論が曖昧
  let result = convert(input); // どの型に変換すべきか不明確
  
  // 修正例: 型パラメータを明示
  let result = convert::<OutputType>(input); // 明示的に型を指定
  ```

- 関数とクロージャの戻り値型
  ```rust
  // 良い例: 戻り値の型を明示的に指定
  fn process_data<T>(data: &[T]) -> Vec<ProcessedResult> {
      // 処理
  }
  
  // クロージャでも同様に明示できる
  let processor = |data: &[f64]| -> Vec<f64> {
      data.iter().map(|&x| x * 2.0).collect()
  };
  ```

これらのパターンを適用することで、型が原因のコンパイルエラーを減らし、コードの意図を明確に伝えることができます。特に数学的計算や複雑なジェネリックコードを扱う場合は、型アノテーションを積極的に活用すべきです。

### 2.5.4 タプル型クエリの利用

ECSシステムでは、`(Entity, &ComponentType)`のようなタプル型クエリを使用することで、エンティティとそのコンポーネントを同時に取得できます。タプル型クエリを使用する際は、以下のパターンに注意してください：

- タプル型クエリの基本構文
  ```rust
  // 良い例: タプル型クエリの使用
  let query = world.query::<(Entity, &NetworkComponent)>();
  for (entity, network) in query.iter(world) {
      // entityとnetworkを使用した処理
  }
  ```

- タプル型と専用クエリメソッドの使用
  ```rust
  // 推奨パターン: query_tupleを使用して型エラーを回避
  let mut base_query = world.query_tuple::<NetworkComponent>();
  let query = base_query.filter(|_, network| network.is_synced && network.is_remote);
  
  for (entity, network) in query.iter(world) {
      // entityとnetworkを使用した処理
  }
  ```

- query_entitiesの戻り値の適切な処理
  ```rust
  // 良い例: query_entitiesの戻り値はEntityのVecなので、Entityとして処理
  for entity in world.query_entities::<NetworkComponent>() {
      // entityを処理
  }
  
  // 避けるべき例: query_entitiesの戻り値をタプルパターンで分解しようとする
  for (entity, _) in world.query_entities::<NetworkComponent>() { // エラー
      // ...
  }
  ```

- 通常のコンポーネントクエリとの使い分け
  ```rust
  // 単一コンポーネントへのクエリ
  let query = world.query::<PositionComponent>();
  for (entity, position) in query.iter(world) {
      // positionを使用した処理
  }
  
  // エンティティとコンポーネントの明示的な関連付け
  let query = world.query::<(Entity, &PositionComponent)>();
  for (entity, position) in query.iter(world) {
      // entityとpositionの関連が明示的
  }
  ```

- タプル型に対するトレイト境界制約の理解
  ```rust
  // 問題のあるコード: (Entity, &NetworkComponent)はComponentトレイトを実装していない
  let query = world.query::<(Entity, &NetworkComponent)>();
  query.filter(|_, network| network.is_synced); // エラー: (Entity, &NetworkComponent)はComponentではない
  
  // 正しいコード: query_tupleメソッドを使用
  let mut base_query = world.query_tuple::<NetworkComponent>();
  let query = base_query.filter(|_, network| network.is_synced);
  
  // または、イテレータのfilterメソッドを使用
  let query = world.query::<(Entity, &NetworkComponent)>();
  for (entity, network) in query.iter(world).filter(|(_, network)| network.is_synced) {
      // フィルタリングされたエンティティとコンポーネントを処理
  }
  ```

- タプル型クエリの変数宣言時のmutキーワード
  ```rust
  // 問題のあるコード: base_queryをimmutableで宣言
  let base_query = world.query_tuple::<NetworkComponent>();
  let query = base_query.filter(|_, network| network.is_synced); // エラー: filterはmutを必要とする
  
  // 正しいコード: base_queryをmutableで宣言
  let mut base_query = world.query_tuple::<NetworkComponent>();
  let query = base_query.filter(|_, network| network.is_synced); // OK
  ```

- タプル型クエリでの複数のコンポーネントアクセス時の注意点
  ```rust
  // 問題のあるパターン: 一度に複数のコンポーネントにアクセス
  for (entity, network) in query.iter(world) {
      let position = world.get_component::<PositionComponent>(entity);
      let velocity = world.get_component::<VelocityComponent>(entity);
      
      // 特定の条件下で処理
      if let (Some(pos), Some(vel)) = (position, velocity) {
          // posとvelを使った処理
      }
  }
  
  // 良いパターン: 事前にデータを収集してから処理
  let mut entities_to_process = Vec::new();
  for (entity, network) in query.iter(world) {
      if let (Some(position), Some(velocity)) = (
          world.get_component::<PositionComponent>(entity),
          world.get_component::<VelocityComponent>(entity)
      ) {
          entities_to_process.push((entity, position.clone(), velocity.clone()));
      }
  }
  
  // 収集したデータを処理
  for (entity, position, velocity) in entities_to_process {
      // 安全に処理を実行
  }
  ```

タプル型クエリはECSシステムの柔軟性を高める機能ですが、型システムとの相互作用に注意が必要です。適切なパターンを使用することで、型エラーを回避し、コードの可読性を向上させることができます。特に`query_tuple`メソッドを使用することで、タプル型に対するComponentトレイト境界の制約を回避できます。

### 2.5.5 カプセル化と適切なアクセサーの使用

Rustではカプセル化された構造体のプライベートフィールドにアクセスする場合、適切なアクセサーメソッドを使用する必要があります。プライベートフィールドへの直接アクセスは、たとえ同じモジュール内でも、コードの整合性や将来的な変更に影響を与える可能性があります。

- プライベートフィールドへのアクセスパターン
  ```rust
  // 問題のあるコード: プライベートフィールドに直接アクセス
  let entity = Entity {
      id: entity_id,
      generation: 0, // プライベートフィールドへの直接アクセス
  };
  
  // 正しいコード: 公開メソッドやコンストラクタを使用
  let entity = Entity::new();
  // または
  let entity = Entity::from_id(entity_id);
  ```

- アクセサーメソッドを使った値の取得
  ```rust
  // 問題のあるコード: プライベートフィールドに直接アクセス
  let entity_id = entity.id; // エラー: `id`はプライベート
  
  // 正しいコード: アクセサーメソッドを使用
  let entity_id = entity.id(); // パブリックメソッドを使用
  ```

- 構造体を再構築する場合のパターン
  ```rust
  // 問題のあるコード: プライベートフィールドから直接構造体を再構築
  let new_entity = Entity {
      id: original_entity.id, // エラー: `id`はプライベート
      generation: original_entity.generation, // エラー: `generation`はプライベート
  };
  
  // 正しいコード: クローンメソッドか、適切なファクトリメソッドを使用
  let new_entity = original_entity.clone();
  // または
  let new_entity = Entity::from_parts(original_entity.id(), original_entity.generation());
  ```

- エンティティIDの取得と保存
  ```rust
  // 良い例: エンティティを識別するために公開メソッドを使用
  let entity_id = entity.id();
  saved_entities.insert(entity_id);
  
  // 良い例: エンティティ全体を保存する場合はCloneトレイトを使用
  let entity_copy = entity.clone();
  entities.push(entity_copy);
  ```

- 構造体のフィールドが変更された場合の影響範囲の最小化
  ```rust
  // 問題のあるパターン: 内部構造に依存するコード
  fn get_entity_debug_string(entity: &Entity) -> String {
      format!("Entity(id={}, gen={})", entity.id, entity.generation) // エラー: プライベートフィールド
  }
  
  // 良いパターン: 公開インターフェースに依存するコード
  fn get_entity_debug_string(entity: &Entity) -> String {
      format!("Entity(id={}, gen={})", entity.id(), entity.generation())
  }
  ```

- 構造体の変更に強いテストコード
  ```rust
  // 問題のあるテスト: 内部構造に依存
  #[test]
  fn test_entity_creation() {
      let entity = Entity::new();
      assert_eq!(entity.generation, 0); // エラー: プライベートフィールド
  }
  
  // 良いテスト: 公開インターフェースを使用
  #[test]
  fn test_entity_creation() {
      let entity = Entity::new();
      assert_eq!(entity.generation(), 0);
  }
  ```

カプセル化はRustのモジュールシステムの重要な側面であり、適切なパブリックインターフェースを使用することで、コードの保守性と将来の変更への耐性が向上します。構造体の内部実装に依存せず、公開されたメソッドやコンストラクタを使用することで、コードは堅牢かつ安全になります。特に、ECSシステムのようなエンティティを多用するコードでは、この原則に従うことで予期しないエラーを防ぐことができます。

### 2.6 Result値を無視しない

Rustでは、`Result<T, E>`型を返す関数の結果を明示的に処理するのがベストプラクティスです。結果を無視すると、エラーが発生しても見落とされ、バグの原因になる可能性があります。

- **Result値を処理するベストプラクティス**
  - 関数がエラーを返せる場合、その結果は常に処理または伝播すべきです
  - エラーが発生しそうな場合は、適切に処理し、ユーザーに情報を提供してください
  - 結果を完全に無視する場合は、`let _ = ...`を使って意図的であることを明示してください

#### 例：

👎 **悪い例**
```rust
// Result値を無視して潜在的なエラーを見落とす
self.context.translate(x, y);
self.context.rotate(angle);
self.context.scale(scale_x, scale_y);
```

👍 **良い例 - エラー処理**
```rust
// Resultを?演算子で伝播
self.context.translate(x, y)?;
self.context.rotate(angle)?;
self.context.scale(scale_x, scale_y)?;
```

👍 **良い例 - 意図的に無視する場合**
```rust
// エラーを意図的に無視する場合は明示的に
let _ = self.context.translate(x, y);
let _ = self.context.rotate(angle);
let _ = self.context.scale(scale_x, scale_y);
```

#### Canvas APIでの実装例：

`Renderer`の`draw_sprite`メソッドでは、Canvas APIからのResult値を適切に処理しています：

```rust
// キャンバスの状態変更操作のResult値を無視せず、明示的に処理
let _ = self.context.translate(screen_x, screen_y);

if sprite.rotation != 0.0 {
    let _ = self.context.rotate(sprite.rotation);
}

if sprite.scale_x != 1.0 || sprite.scale_y != 1.0 {
    let _ = self.context.scale(sprite.scale_x, sprite.scale_y);
}

if sprite.opacity != 1.0 {
    let _ = self.context.set_global_alpha(sprite.opacity);
}

// エラー伝播が必要な操作は?演算子を使用
self.context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
    &image,
    sprite.source_x,
    sprite.source_y,
    sprite.source_width,
    sprite.source_height,
    -sprite.width * sprite.pivot_x,
    -sprite.height * sprite.pivot_y,
    sprite.width,
    sprite.height,
)?;
```

このようにして、キャンバス操作のエラーを見落とすことなく、適切に処理されています。

## 3. テスト

### 3.1 ユニットテスト
- テストモジュールの配置
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      use wasm_bindgen_test::*;

      #[wasm_bindgen_test]
      fn test_entity_creation() {
          // ...
      }
  }
  ```

### 3.2 テストカバレッジ
- 主要な機能のテスト
- エッジケースのテスト
- エラーパスのテスト

## 4. パフォーマンス

### 4.1 メモリ管理
- 適切なデータ構造の選択
  ```rust
  // 高速なルックアップが必要な場合
  use std::collections::HashMap;
  
  // 連続したメモリが必要な場合
  use std::vec::Vec;
  ```
- メモリリークの防止
  ```rust
  impl Drop for Resource {
      fn drop(&mut self) {
          // リソースの解放
      }
  }
  ```

### 4.2 所有権とメソッドチェーン
- メソッドチェーンでの所有権移動に注意
  ```rust
  // 問題のあるコード: with_positionメソッドが所有権を消費する場合
  snapshot.with_position([x, y, z]); // snapshotの所有権が移動する
  snapshot.with_velocity([vx, vy, vz]); // エラー: 既に移動したsnapshotを使用

  // 改善策1: 直接フィールドに代入
  let pos = [x, y, z];
  snapshot.position = Some(pos);
  snapshot.velocity = Some([vx, vy, vz]);

  // 改善策2: メソッドが&mut selfを取るように設計
  snapshot.set_position([x, y, z]); // 所有権を移動しない
  snapshot.set_velocity([vx, vy, vz]); // OK
  ```

- EntityIdのような不透明な型を安全に扱う
  ```rust
  // EntityIdをu64に安全に変換
  let entity_id = entity.id();
  let id_value = match format!("{}", entity_id).strip_prefix("Entity(").and_then(|s| s.strip_suffix(")")) {
      Some(id_str) => id_str.parse::<u64>().unwrap_or(0),
      None => 0,
  };
  ```

- 値の変換時は適切なエラーハンドリング
  ```rust
  // u64からu32への安全な変換
  let u32_value = u64_value.try_into().unwrap_or(0);
  ```

### 4.3 最適化
- プロファイリングの実施
- ボトルネックの特定と改善
- キャッシュの活用

## 5. セキュリティ

### 5.1 入力検証
- ユーザー入力の検証
  ```rust
  fn validate_input(input: &str) -> Result<(), ValidationError> {
      if input.is_empty() {
          return Err(ValidationError::EmptyInput);
      }
      // ...
  }
  ```

## 6. トラブルシューティング

### 6.1 物理エンジン関連の修正
- `PhysicsEntity` には `Clone` トレイトを実装済み (#[derive(Clone)]を追加済み)
- `SpatialGrid` と `CollisionFilter` にも `Clone` トレイトを実装済み (#[derive(Clone)]を追加済み)
- `SpatialGrid` の `add_entity` メソッドは、正しくは `insert_entity` であることに注意して修正済み
- `CollisionFilter` の `set_entity_category` および `set_entity_mask` メソッドは、正しくはそれぞれ `set_category` および `set_mask` であり修正済み
- `PhysicsStep.update()` は `(usize, f64)` のタプルを返すため、それに応じた処理が必要 (タプルからステップ数を取り出して処理するよう修正済み)
- 衝突ペア（collision pairs）はタプル `(u32, u32)` であり、これらの値は `pair.0` および `pair.1` としてアクセスするよう修正済み
- `generate_collision_pairs` 関数の呼び出し時、`collision_filter` 引数は `Option<CollisionFilter>` 型として渡す必要があるため、`&Some(collision_filter.clone())` として渡すよう修正済み
- 衝突解決時に同時に2つのエンティティを可変借用できないため、エンティティを順番に処理するように修正済み

### 6.2 ECS関連
- `query` メソッドと関連機能の実装には注意が必要
- コンポーネントとシステムの相互作用では、適切な型の使用を確認すべき

### 6.3 Resourceトレイトの実装

- Resourceトレイトは正しく実装し、必須メソッドを忘れないこと
  ```rust
  // 良い例: Resourceトレイトの完全な実装
  impl Resource for PhysicsWorld {
      fn as_any(&self) -> &dyn std::any::Any {
          self
      }
      
      fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
          self
      }
  }
  
  // 避けるべき例: 不完全な実装
  impl Resource for PhysicsWorld {} // コンパイルエラー: as_any と as_any_mut が実装されていない
  ```

- 派生マクロを使用してResourceトレイトを自動実装する
  ```rust
  // #[derive(Resource)]マクロを使用した実装
  #[derive(Debug, Clone, Resource)]
  pub struct GameState {
      // フィールド
  }
  ```

- Anyトレイトへのキャストの重要性
  ```rust
  // ResourceManagerからリソースを取得する際の型キャスト
  let physics_world = resource_manager.get::<PhysicsWorld>().unwrap();
  
  // 内部的には以下のような処理が行われている
  fn get<T: Resource>(&self) -> Option<&T> {
      self.resources.get(&TypeId::of::<T>())
          .and_then(|resource| resource.as_any().downcast_ref::<T>())
  }
  ```

- リソースアクセスのスレッド安全性
  ```rust
  // 非Wasm環境ではSendとSyncが必要
  #[cfg(not(target_arch = "wasm32"))]
  fn get_resource<T: 'static + Send + Sync + Resource>(&self) -> Option<&T>;
  
  // Wasm環境では不要
  #[cfg(target_arch = "wasm32")]
  fn get_resource<T: 'static + Resource>(&self) -> Option<&T>;
  ```

### 7. WebAssembly対応

WebAssembly環境では、通常のRustアプリケーションとは異なる制約や考慮事項があります。

### 7.1 スレッドセーフティ制約の緩和

- WebAssemblyはシングルスレッド環境で動作するため、`Send`と`Sync`トレイトの要件を適切に緩和する
  ```rust
  // 通常環境とWasm環境で異なる実装を提供
  #[cfg(not(target_arch = "wasm32"))]
  pub trait Resource: 'static + Send + Sync + Any {
      // ...
  }

  #[cfg(target_arch = "wasm32")]
  pub trait Resource: 'static + Any {
      // ...
  }
  ```

### 7.2 JavaScriptとの相互運用

- JavaScriptオブジェクトを扱う際は、所有権の移動に注意
  ```rust
  // JavaScriptの値はコピーではなく参照として扱う
  let canvas_context = document
      .get_element_by_id("game-canvas")?
      .dyn_into::<HtmlCanvasElement>()?
      .get_context("2d")?
      .unwrap()
      .dyn_into::<CanvasRenderingContext2d>()?;
  ```

- エラーハンドリングにおいては`JsValue`を適切に変換
  ```rust
  pub fn init() -> Result<(), JsValue> {
      // エラーを適切にJsValueに変換
      let config = get_config().map_err(|e| JsValue::from_str(&e.to_string()))?;
      Ok(())
  }
  ```

### 7.3 条件付きコンパイル

- 環境に応じた条件付きコンパイルを活用
  ```rust
  // WebAssembly環境専用の実装
  #[cfg(target_arch = "wasm32")]
  fn init_web_api() -> Result<(), JsValue> {
      // Webブラウザ専用の初期化コード
  }

  // 非WebAssembly環境専用の実装
  #[cfg(not(target_arch = "wasm32"))]
  fn init_native_api() -> Result<(), String> {
      // ネイティブ環境専用の初期化コード
  }
  ```

### 7.4 メモリ管理

- WebAssembly環境ではメモリ制約に注意し、大きなメモリアロケーションを避ける
  ```rust
  // 大きな配列を作る代わりにイテレータを使用
  fn process_large_data() {
      // 悪い例: 大きな配列をメモリに保持
      // let large_array = vec![0; 1_000_000];
      
      // 良い例: イテレータを使用して少しずつ処理
      (0..1_000_000).map(|i| i * 2).for_each(|value| {
          process_value(value);
      });
  }
  ```

### 7.5 リソースの取得と利用

- 正しいリソース型を使用する
  ```rust
  // 問題のあるコード: リソースとして登録されていない型を直接取得しようとしている
  let input_system = world.get_resource_mut::<InputSystem>();  // エラー: InputSystemはResourceトレイトを実装していない
  
  // 正しいコード: 適切なリソース型を取得してから目的のデータにアクセスする
  let input_system = world.get_resource_mut::<InputResource>()
      .map(|input_resource| &mut input_resource.system);
  ```

- 複合リソースの設計パターン
  ```rust
  // 推奨される設計: 複数の関連コンポーネントを一つのリソースにまとめる
  pub struct InputResource {
      /// 入力状態
      pub state: InputState,
      /// 入力システム
      pub system: InputSystem,
  }
  
  impl Resource for InputResource {
      fn as_any(&self) -> &dyn std::any::Any {
          self
      }
      
      fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
          self
      }
  }
  ```

- リソースからのデータ取得パターン
  ```rust
  // 良い例: リソースから必要なデータだけを取り出す関数
  fn get_input_system(&mut self) -> Option<&mut InputSystem> {
      self.world.get_resource_mut::<InputResource>()
          .map(|input_resource| &mut input_resource.system)
  }
  ```

この設計パターンを採用することで、リソースの組織化と型安全性を両立させることができます。システムコンポーネントは適切なリソースラッパー内にカプセル化し、直接WorldにResourceとして登録するのは避けるべきです。

## 8. マインスイーパープロジェクト固有の品質要件

マルチプレイヤーマインスイーパープロジェクトでは、以下の品質要件に特に注意する必要があります：

### 8.1 メッセージ型の一貫性

- クライアントとサーバー間のすべてのメッセージは`message_protocol.md`で定義された構造に厳密に従うこと
- メッセージのシリアライズ/デシリアライズ時に型の不一致が発生しないよう注意すること
- 特に以下のメッセージタイプに注意：
  - `ChordAction`と`chord_action`メソッドの名前の一貫性
  - 座標情報は`Position`構造体ではなく`x`と`y`の直接使用に統一
  - プレイヤーIDは`PlayerId`型ではなく`String`型に統一

### 8.2 WebAssembly固有の最適化

- メモリ使用量の最小化：大きな配列よりイテレータの使用を優先
- DOM操作の最小化：キャンバスの再描画は変更された部分のみに限定
- JavaScript側とのやり取りは最小限に抑え、Rust側でロジックを完結させる

### 8.3 ECSシステムの一貫した使用

- すべてのゲームエンティティはECSパターンに基づいて実装
- コンポーネントは明確に定義され、適切に分離されていること
- システムはコンポーネントに対して操作を行い、直接エンティティを変更しない

### 8.4 テスト戦略

- ネットワーク通信部分は特に入念にテストすること
- WebSocketメッセージの送受信テスト
- 異常系（切断、再接続など）のテスト
- マルチプレイヤー同期のテスト

このガイドラインを遵守することで、マルチプレイヤーマインスイーパーのコードの品質と保守性を高め、バグの発生を最小限に抑えることができます。

## 9. WebAssemblyとCanvas API

### 9.1 HTML5 Canvas APIの正確なメソッド名の使用

- Canvas APIのメソッド名を正確に使用することが重要です
  ```rust
  // 間違い: 存在しないメソッド名
  self.context.set_fill_style_with_str("#1a75ff");
  
  // 正解: 正確なメソッド名
  self.context.set_fill_style_str("#1a75ff");
  ```

### 9.2 Canvas APIの引数パターン

- `fill_text`メソッドの引数は正確に3つ必要（テキスト、x座標、y座標）
  ```rust
  // 間違い: 余分なNoneパラメータ
  self.context.fill_text(
      "Score: 0",
      10.0,
      20.0,
      None, // 余分なパラメータ
  )?;
  
  // 正解: 正確な引数数
  self.context.fill_text(
      "Score: 0",
      10.0,
      20.0,
  )?;
  ```

### 9.3 Canvas変換操作のエラーハンドリング

- `translate`, `rotate`, `scale`などの操作は`Result`を返すため、エラーハンドリングが必要
  ```rust
  // 間違い: 返り値を無視
  self.context.translate(screen_x, screen_y);
  
  // 正解: エラーハンドリングを追加
  self.context.translate(screen_x, screen_y)?;
  
  // または無視する場合は明示的に
  let _ = self.context.translate(screen_x, screen_y);
  ```

- 描画メソッドでの実装例
  ```rust
  pub fn draw_sprite(&self, sprite_id: &str, x: f64, y: f64) -> Result<(), JsValue> {
      // ...前処理...
      
      // 描画変換を適用
      self.context.save();
      
      // 位置設定 - Resultを明示的に処理
      let _ = self.context.translate(screen_x, screen_y);
      
      // 回転設定 - 条件付きで適用し、Resultを処理
      if sprite.rotation != 0.0 {
          let _ = self.context.rotate(sprite.rotation);
      }
      
      // スケール設定 - 条件付きで適用し、Resultを処理
      if sprite.scale_x != 1.0 || sprite.scale_y != 1.0 {
          let _ = self.context.scale(sprite.scale_x, sprite.scale_y);
      }
      
      // 描画処理...
      
      // 状態を復元
      self.context.restore();
      
      Ok(())
  }
  ```

- `Result`を明示的に無視する場合の注釈例
  ```rust
  // 良い例: Resultを明示的に無視する理由をコメントで説明
  let _ = self.context.translate(screen_x, screen_y); // translateの失敗はここでは重要ではない
  
  // 悪い例: 返り値が無視される理由が不明
  self.context.translate(screen_x, screen_y); // コンパイラ警告の原因になる
  ```

- 実際のプロジェクトでの修正例（src/rendering/mod.rs）
  ```rust
  // 修正前：返り値（Result）を無視している
  self.context.translate(screen_x, screen_y);
  self.context.rotate(sprite.rotation);
  self.context.scale(sprite.scale_x, sprite.scale_y);
  
  // 修正後：返り値を明示的に処理
  let _ = self.context.translate(screen_x, screen_y);
  if sprite.rotation != 0.0 {
      let _ = self.context.rotate(sprite.rotation);
  }
  if sprite.scale_x != 1.0 || sprite.scale_y != 1.0 {
      let _ = self.context.scale(sprite.scale_x, sprite.scale_y);
  }
  ```

これにより、コンパイラ警告が解消され、意図が明確になります。Canvas API操作の`Result`を無視する場合は、常に`let _ =`パターンを使用して明示的に処理することを推奨します。

### 9.4 WebAssembly固有のAPIバインディングの注意点

- web-sysクレートのAPIバインディングはJavaScriptの命名規則と異なる場合がある
- エラーメッセージの提案を確認し、正しいRust側のメソッド名を使用すること
- API変更には互換性が保証されていないことが多いため、バージョンアップ時に注意が必要

### 9.5 Canvas操作のパフォーマンス最適化

- 状態変更が多い処理はブロックで囲み、`save()`と`restore()`で状態管理を行う
  ```rust
  // 良い例: 状態変更を局所化
  self.context.save();
  // 複数の状態変更
  self.context.set_fill_style_str("#ff0000");
  self.context.set_font("16px Arial");
  // 描画操作
  self.context.fill_text("Hello", 10.0, 20.0)?;
  // 状態を元に戻す
  self.context.restore();
  ```

これらのガイドラインに従うことで、WebAssemblyアプリケーションにおけるCanvas APIの使用に関連する問題を防ぎ、より堅牢なコードを作成できます。

## 10. 実装例

### 10.1 JavaScript/WebAssembly連携のための実装パターン

#### グローバルゲームインスタンスの管理
WASMでは、JavaScriptとの通信において効率的なグローバルなインスタンス管理が重要です。以下は実際の実装例です：

```rust
// グローバルインスタンス管理
thread_local! {
    static NETWORK_CLIENTS: RefCell<HashMap<String, Rc<RefCell<network::client::NetworkClient>>>> = 
        RefCell::new(HashMap::new());
    static GAME_INSTANCES: RefCell<HashMap<String, Weak<RefCell<GameInstance>>>> = 
        RefCell::new(HashMap::new());
    static GAME_INSTANCE: RefCell<Option<Rc<RefCell<GameInstance>>>> = RefCell::new(None);
}
```

この実装では、複数の種類のグローバル変数を管理しています：
- `NETWORK_CLIENTS`: 複数のネットワーククライアントを文字列キーで管理
- `GAME_INSTANCES`: 複数のゲームインスタンスを文字列キーで管理（弱参照を使用）
- `GAME_INSTANCE`: 現在アクティブなゲームインスタンスへの参照

#### JavaScript向けのエクスポート関数
JavaScriptから呼び出し可能な関数は、グローバルなゲームインスタンスを安全に利用します：

```rust
#[wasm_bindgen]
pub fn update_mouse_position(x: f32, y: f32) -> Result<(), JsValue> {
    // ゲームインスタンスが初期化されていない場合はエラー
    GAME_INSTANCE.with(|instance| {
        if let Some(instance_rc) = &*instance.borrow() {
            let mut game = instance_rc.borrow_mut();
            // InputResourceを取得して更新
            if let Some(input_resource) = game.world.get_resource_mut::<input::InputResource>() {
                input_resource.set_mouse_position(x, y);
                Ok(())
            } else {
                Err(JsValue::from_str("InputResourceが見つかりません"))
            }
        } else {
            Err(JsValue::from_str("ゲームが初期化されていません"))
        }
    })
}
```

この実装の特徴：
1. エラーハンドリング: ゲームインスタンスやリソースの不在を適切に処理
2. 型安全性: JavaScript呼び出し側と適切に型変換
3. 参照カウント: `Rc<RefCell<T>>`パターンを利用して安全な参照管理
4. 明確なエラーメッセージ: 問題の原因を理解しやすいメッセージを返す

#### リソース管理のヘルパーメソッド
リソースの更新を簡潔に行うためのヘルパーメソッド：

```rust
/// マウス位置を設定
pub fn set_mouse_position(&mut self, x: f32, y: f32) {
    // delta_timeは小さな値を使用（実際の時間は不明なため）
    self.state.update_mouse_position(x, y, 0.016);
}
```

この実装は、低レベルの操作（delta_timeパラメータを必要とするメソッド）を、より使いやすい高レベルAPIとして提供しています。これはAPIデザインの良い例で、利用者（コード内の他の部分）に不要な詳細を隠蔽しています。

#### ネットワーク状態監視

```rust
// NetworkStatusMonitorの実装例
impl System for NetworkStatusMonitor {
    fn name(&self) -> &'static str {
        "NetworkStatusMonitor"
    }
    
    fn phase(&self) -> SystemPhase {
        SystemPhase::NetworkSync
    }
    
    fn priority(&self) -> SystemPriority {
        SystemPriority::new(10) // ネットワーク状態は早めに更新
    }

    // 良い例: 未使用パラメータにアンダースコアプレフィックスを付ける
    fn run(&mut self, world: &mut World, _resources: &mut ResourceManager, _delta_time: f32) -> Result<(), JsValue> {
        let now = Date::now();
        
        // リソースは使わないので_resourcesとしている
        // delta_timeも使わないので_delta_timeとしている
        
        // ... その他の処理 ...
        Ok(())
    }
}
```

この実装は、ネットワーク状態の変化を監視し、必要に応じてゲームの状態を調整するために使用されます。

## 2. コード警告の管理

### 2.1 最近の警告修正

#### 2023-XX-XX: 未使用コードの警告修正
- `src/ecs/mod.rs`内の未使用の`TypeIdExt`特性に`_`プレフィックスを追加して警告を抑制しました。
  ```rust
  // TypeIdから型名を取得するための拡張トレイト
  trait _TypeIdExt {
      fn type_name(&self) -> &'static str;
  }
  
  impl _TypeIdExt for std::any::TypeId {
      fn type_name(&self) -> &'static str {
          std::any::type_name::<Self>()
      }
  }
  ```

#### 2023-XX-XX: NetworkClient構造体の未使用フィールド修正
- `src/network/client.rs`内の`NetworkClient`構造体の未使用フィールドに`_`プレフィックスを追加して警告を抑制しました。
  ```rust
  pub struct NetworkClient {
      // ...
      /// エンティティスナップショットキャッシュ
      _entity_snapshots: HashMap<u32, Vec<EntitySnapshot>>,
      /// 他プレイヤーのプレイヤーデータ
      _players: HashMap<u32, PlayerData>,
      // ...
  }
  ```

#### 2023-XX-XX: CompressionStats構造体の未使用フィールド修正
- `src/network/sync.rs`内の`CompressionStats`構造体の未使用フィールドに`_`プレフィックスを追加して警告を抑制しました。
  ```rust
  pub struct CompressionStats {
      // ...
      /// デルタ圧縮で省略されたフィールド数
      _delta_skipped_fields: usize,
      /// マスキングで省略されたフィールド数
      _masked_fields: usize,
      /// 量子化された値の数
      _quantized_values: usize,
  }
  ```

#### 2023-XX-XX: InterpolationSystem構造体の未使用フィールド修正
- `src/network/prediction.rs`内の`InterpolationSystem`構造体の未使用フィールドに`_`プレフィックスを追加して警告を抑制しました。
  ```rust
  pub struct InterpolationSystem {
      /// 補間バッファの時間（ミリ秒）
      _buffer_time: f64,
      /// 最後の更新時刻
      last_update: f64,
  }
  ```

### 2.2 今後の警告解決計画

以下の未使用フィールドの警告が残っており、優先度順に修正を進める予定です：

1. **NetworkMessage構造体の未使用フィールド**（優先度：高）
   - ✅ `message_type`と`timestamp`フィールドに`#[allow(dead_code)]`属性を追加して対応済み

2. **BandwidthUsage構造体の未使用フィールド**（優先度：中）
   - `peak_bandwidth`フィールドが使用されていない
   - 予定対応: 帯域幅モニタリングで使用するか、`#[allow(dead_code)]`属性を追加する

3. **MouseCursorComponent参照の問題**（優先度：高）
   - ✅ `src/game/cursor/rendering.rs`の38行目で、`&MouseCursorComponent`から`&`を取り除いて解決済み
   - ECSの仕様に従って、コンポーネントは参照ではなく直接渡す必要がある

4. **NetworkClient関連の問題**（優先度：高）
   - ✅ `NetworkClient`に`Resource`トレイトを手動実装して解決済み
   - `as_any`と`as_any_mut`メソッドを実装し、適切なインポートを追加

5. **InputResourceのメソッド不足**（優先度：中）
   - `is_mouse_in_canvas`メソッドが`InputResource`に存在しない
   - 予定対応: メソッドを追加するか、代替の方法で実装

6. **World::get_system_mutメソッドの不在**（優先度：中）
   - `get_system_mut`メソッドが`World`に存在しない
   - 予定対応: システムへのアクセス方法を代替実装する

7. **その他の警告**（優先度：低）
   - 未使用インポート: `wasm_bindgen::prelude::*`など
   - 非推奨メソッド: `set_fill_style`などの使用
   - 予定対応: 段階的に対処する
   
これらの警告を解決することで、コードの品質と保守性を向上させ、将来的なバグの発生リスクを低減します。

### 1.7 未使用のフィールド (dead code) の処理

構造体やクラス内で定義したが使用していないフィールドがある場合、Rustコンパイラは「dead code」警告を出します。これらのフィールドを適切に処理することは、コードの品質と保守性を維持するために重要です。

未使用フィールドを処理するには、以下の方法があります：

1. **フィールドに使用目的がある場合**：そのフィールドを使用するコードを実装してください。
2. **将来的に使用する予定のフィールド**：`#[allow(dead_code)]`属性を付与して、コンパイラ警告を抑制します。
3. **使用しないことが確定しているフィールド**：そのフィールドを削除してコードをクリーンに保ちます。

#### 例：

👎 **悪い例**
```rust
pub struct NetworkMessage {
    pub message_type: MessageType,  // 警告: フィールドが使われていない
    pub timestamp: f64,            // 警告: フィールドが使われていない
    pub data: Vec<u8>,
}
```

👍 **良い例（将来的に使用する予定のフィールド）**
```rust
pub struct NetworkMessage {
    #[allow(dead_code)]
    pub message_type: MessageType,  // 将来使用予定のため警告を抑制
    #[allow(dead_code)]
    pub timestamp: f64,            // 将来使用予定のため警告を抑制
    pub data: Vec<u8>,
}
```

👍 **より良い例（不要なフィールドを削除）**
```rust
pub struct NetworkMessage {
    pub data: Vec<u8>,  // 使用するフィールドのみ残す
}
```

#### 実際の実装例：

`BandwidthUsage`構造体では、現在使用していないが将来的に必要になる可能性がある`peak_bandwidth`フィールドに`#[allow(dead_code)]`属性を使用しています：

```rust
pub struct BandwidthUsage {
    pub recent_bytes_sent: VecDeque<(Instant, u64)>,
    pub recent_bytes_received: VecDeque<(Instant, u64)>,
    #[allow(dead_code)]
    pub peak_bandwidth: f64,  // 将来的に使用する予定のフィールド
    pub estimated_available_bandwidth: f64,
    pub target_usage_ratio: f64,
}
```

この属性を使うことで：
- コンパイラ警告を抑制できる
- 将来的な利用のためにフィールドを保持する意図が明確になる
- 設計上必要なフィールドであることを示せる

注意点：
- `#[allow(dead_code)]`属性は必要最小限にとどめ、本当に将来必要になるフィールドにのみ使用する
- 完全に不要なフィールドは削除して、コード全体をシンプルに保つ
- ドキュメントコメントで属性の使用理由を説明すると、より意図が明確になる

## 5. 最近の対応履歴

### GameStateとResourceトレイトの実装

- **問題点**: GameStateがResourceトレイトを実装していなかったため、リソースとして使用できなかった
- **原因**: GameStateをECSのリソースとして使用するためには、Resourceトレイトを明示的に実装する必要がある
- **対応**:
  1. `GameState`に`Resource`トレイトを実装
  ```rust
  // Resourceトレイトの実装
  impl Resource for GameState {
      fn as_any(&self) -> &dyn std::any::Any {
          self
      }
  
      fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
          self
      }
  }
  ```
  2. プライベートフィールド`current_state`へのアクセサメソッドを追加
  ```rust
  /// 現在の状態を取得します。
  pub fn get_state(&self) -> GameStateType {
      self.current_state
  }
  
  /// 現在の状態を設定します。
  pub fn set_state(&mut self, state: GameStateType) {
      log::info!("ゲーム状態を変更: {:?} -> {:?}", self.current_state, state);
      self.current_state = state;
  }
  ```
  3. 状態変更コードをアクセサメソッドを使用するように修正
  ```rust
  // 以前のコード（エラー）
  *game_state = game::state::GameState::Solitaire;
  
  // 修正後のコード
  game_state.set_state(game::state::GameStateType::Solitaire);
  ```

### 型の混同と解決方法

- **問題点**: `GameState`と`GameStateType`の混同によるエラー
- **対応**:
  - 明確に型を区別して使用する
  - `GameState`は構造体（Resourceとして実装）
  - `GameStateType`は列挙型（ゲームの状態を表現）
  - アクセサメソッドを使用して適切に状態を操作する
