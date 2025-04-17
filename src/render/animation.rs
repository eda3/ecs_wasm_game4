use wasm_bindgen::prelude::*;
use crate::ecs::world::World;
use crate::ecs::resources::ResourceManager;
use crate::ecs::component::{Transform, Renderable};
use crate::ecs::system::{System, SystemPhase, SystemPriority};
use crate::utils::Vec2;
use std::collections::HashMap;
use crate::constants::ANIMATION_DURATION;

/// アニメーション種類
#[derive(Clone, Debug)]
pub enum AnimationType {
    /// 移動アニメーション
    Move {
        start_pos: Vec2,
        end_pos: Vec2,
        duration: f64,    // ミリ秒
        easing: EasingType,
    },
    /// 回転アニメーション
    Rotate {
        start_angle: f64,
        end_angle: f64,
        duration: f64,
        easing: EasingType,
    },
    /// 拡大縮小アニメーション
    Scale {
        start_scale: Vec2,
        end_scale: Vec2,
        duration: f64,
        easing: EasingType,
    },
    /// フェードアニメーション
    Fade {
        start_opacity: f64,
        end_opacity: f64,
        duration: f64,
        easing: EasingType,
    },
}

/// イージング関数タイプ
#[derive(Clone, Debug)]
pub enum EasingType {
    /// 線形（一定速度）
    Linear,
    /// イーズイン（徐々に加速）
    EaseIn,
    /// イーズアウト（徐々に減速）
    EaseOut,
    /// イーズインアウト（加速して減速）
    EaseInOut,
    /// バウンス（跳ね返るような動き）
    Bounce,
    /// エラスティック（弾むような動き）
    Elastic,
}

/// アニメーション状態
#[derive(Clone, Debug)]
pub struct Animation {
    pub entity_id: usize,
    pub animation_type: AnimationType,
    pub elapsed_time: f64,  // 経過時間（ミリ秒）
    pub completed: bool,
}

impl Animation {
    /// 新しいアニメーションを作成
    pub fn new(entity_id: usize, animation_type: AnimationType) -> Self {
        Self {
            entity_id,
            animation_type,
            elapsed_time: 0.0,
            completed: false,
        }
    }
    
    /// アニメーションの進行度を計算（0.0～1.0）
    pub fn progress(&self) -> f64 {
        let duration = match &self.animation_type {
            AnimationType::Move { duration, .. } => *duration,
            AnimationType::Rotate { duration, .. } => *duration,
            AnimationType::Scale { duration, .. } => *duration,
            AnimationType::Fade { duration, .. } => *duration,
        };
        
        if duration <= 0.0 {
            return 1.0;
        }
        
        let progress = self.elapsed_time / duration;
        if progress >= 1.0 {
            1.0
        } else {
            progress
        }
    }
    
    /// イージング関数を適用した進行度を計算
    pub fn eased_progress(&self) -> f64 {
        let progress = self.progress();
        
        let easing = match &self.animation_type {
            AnimationType::Move { easing, .. } => easing,
            AnimationType::Rotate { easing, .. } => easing,
            AnimationType::Scale { easing, .. } => easing,
            AnimationType::Fade { easing, .. } => easing,
        };
        
        match easing {
            EasingType::Linear => progress,
            EasingType::EaseIn => progress * progress,
            EasingType::EaseOut => 1.0 - (1.0 - progress) * (1.0 - progress),
            EasingType::EaseInOut => {
                if progress < 0.5 {
                    2.0 * progress * progress
                } else {
                    1.0 - (-2.0 * progress + 2.0).powi(2) / 2.0
                }
            },
            EasingType::Bounce => {
                // バウンス関数の実装
                let p = progress;
                if p < 1.0 / 2.75 {
                    7.5625 * p * p
                } else if p < 2.0 / 2.75 {
                    let p = p - 1.5 / 2.75;
                    7.5625 * p * p + 0.75
                } else if p < 2.5 / 2.75 {
                    let p = p - 2.25 / 2.75;
                    7.5625 * p * p + 0.9375
                } else {
                    let p = p - 2.625 / 2.75;
                    7.5625 * p * p + 0.984375
                }
            },
            EasingType::Elastic => {
                // エラスティック関数の実装
                let p = progress;
                (2.0_f64.powf(-10.0 * (1.0 - p)) * (1.0 - p) * (2.0 * std::f64::consts::PI).sin() / 0.3 + 1.0)
            },
        }
    }
    
    /// アニメーションを更新
    pub fn update(&mut self, delta_time: f32) {
        if self.completed {
            return;
        }
        
        // 経過時間を更新
        self.elapsed_time += delta_time as f64 * 1000.0;  // 秒をミリ秒に変換
        
        // 完了判定
        let duration = match &self.animation_type {
            AnimationType::Move { duration, .. } => *duration,
            AnimationType::Rotate { duration, .. } => *duration,
            AnimationType::Scale { duration, .. } => *duration,
            AnimationType::Fade { duration, .. } => *duration,
        };
        
        if self.elapsed_time >= duration {
            self.completed = true;
            self.elapsed_time = duration;
        }
    }
    
    /// アニメーション値を計算
    pub fn calculate_value<T: Copy + std::ops::Add<Output = T> + std::ops::Mul<f64, Output = T>>(
        &self,
        start: T,
        end: T,
    ) -> T {
        let t = self.eased_progress();
        start + (end - start) * t
    }
}

/// アニメーションマネージャー
/// 複数のアニメーションを管理するリソース
#[derive(Default)]
pub struct AnimationManager {
    animations: Vec<Animation>,
    completed_animations: Vec<usize>,  // 完了したアニメーションのインデックス
}

impl AnimationManager {
    /// 新しいアニメーションマネージャーを作成
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
            completed_animations: Vec::new(),
        }
    }
    
    /// アニメーションを追加
    pub fn add_animation(&mut self, animation: Animation) {
        self.animations.push(animation);
    }
    
    /// エンティティを指定した位置に移動するアニメーションを追加
    pub fn move_entity(
        &mut self,
        entity_id: usize,
        end_pos: Vec2,
        start_pos: Option<Vec2>,
        duration: Option<f64>,
        easing: Option<EasingType>,
    ) {
        // デフォルト値の設定
        let duration = duration.unwrap_or(ANIMATION_DURATION);
        let easing = easing.unwrap_or(EasingType::EaseInOut);
        
        let animation = Animation::new(
            entity_id,
            AnimationType::Move {
                start_pos: start_pos.unwrap_or(Vec2::zero()),  // 実際の開始位置はアニメーションシステムで設定
                end_pos,
                duration,
                easing,
            },
        );
        
        self.add_animation(animation);
    }
    
    /// エンティティをフェードイン/アウトするアニメーションを追加
    pub fn fade_entity(
        &mut self,
        entity_id: usize,
        end_opacity: f64,
        start_opacity: Option<f64>,
        duration: Option<f64>,
        easing: Option<EasingType>,
    ) {
        // デフォルト値の設定
        let duration = duration.unwrap_or(ANIMATION_DURATION);
        let easing = easing.unwrap_or(EasingType::EaseInOut);
        
        let animation = Animation::new(
            entity_id,
            AnimationType::Fade {
                start_opacity: start_opacity.unwrap_or(1.0),  // 実際の開始値はアニメーションシステムで設定
                end_opacity,
                duration,
                easing,
            },
        );
        
        self.add_animation(animation);
    }
    
    /// エンティティのアニメーションをすべて削除
    pub fn remove_animations_for_entity(&mut self, entity_id: usize) {
        self.animations.retain(|anim| anim.entity_id != entity_id);
    }
    
    /// すべてのアニメーションを更新
    pub fn update_animations(&mut self, delta_time: f32) {
        self.completed_animations.clear();
        
        // 各アニメーションを更新し、完了したものを記録
        for (i, animation) in self.animations.iter_mut().enumerate() {
            animation.update(delta_time);
            if animation.completed {
                self.completed_animations.push(i);
            }
        }
        
        // 完了したアニメーションを削除（インデックスが大きい順に削除）
        self.completed_animations.sort_by(|a, b| b.cmp(a));
        for &index in &self.completed_animations {
            self.animations.remove(index);
        }
    }
    
    /// エンティティのアニメーションを取得
    pub fn get_animations_for_entity(&self, entity_id: usize) -> Vec<&Animation> {
        self.animations
            .iter()
            .filter(|anim| anim.entity_id == entity_id)
            .collect()
    }
    
    /// アニメーションの数を取得
    pub fn animation_count(&self) -> usize {
        self.animations.len()
    }
}

/// アニメーションシステム
/// エンティティのアニメーションを処理するシステム
pub struct AnimationSystem;

impl AnimationSystem {
    /// 新しいアニメーションシステムを作成
    pub fn new() -> Self {
        Self
    }
}

impl System for AnimationSystem {
    fn name(&self) -> &'static str {
        "AnimationSystem"
    }
    
    fn phase(&self) -> SystemPhase {
        SystemPhase::Update  // 更新フェーズで実行
    }
    
    fn priority(&self) -> SystemPriority {
        SystemPriority::new(50)  // 優先度：更新フェーズの中間
    }
    
    fn run(&mut self, world: &mut World, resources: &mut ResourceManager, delta_time: f32) -> Result<(), JsValue> {
        // アニメーションマネージャーを取得
        let mut animation_manager = match resources.get_mut::<AnimationManager>() {
            Some(manager) => manager,
            None => return Ok(()),  // アニメーションマネージャーがなければ何もしない
        };
        
        // 実行前にアニメーションの開始位置など、初期状態を設定
        for animation in &mut animation_manager.animations {
            if animation.elapsed_time == 0.0 {
                // アニメーションが開始直後の場合、現在の状態を取得して初期値を設定
                match &mut animation.animation_type {
                    AnimationType::Move { start_pos, .. } => {
                        if let Some(transform) = world.get_component::<Transform>(animation.entity_id) {
                            *start_pos = transform.position;
                        }
                    },
                    AnimationType::Rotate { start_angle, .. } => {
                        if let Some(transform) = world.get_component::<Transform>(animation.entity_id) {
                            *start_angle = transform.rotation;
                        }
                    },
                    AnimationType::Scale { start_scale, .. } => {
                        if let Some(transform) = world.get_component::<Transform>(animation.entity_id) {
                            *start_scale = transform.scale;
                        }
                    },
                    AnimationType::Fade { start_opacity, .. } => {
                        if let Some(renderable) = world.get_component::<Renderable>(animation.entity_id) {
                            *start_opacity = renderable.opacity;
                        }
                    },
                }
            }
        }
        
        // 全てのアニメーションを更新
        animation_manager.update_animations(delta_time);
        
        // エンティティごとにアニメーションを適用
        let entity_ids: Vec<usize> = animation_manager
            .animations
            .iter()
            .map(|anim| anim.entity_id)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        
        for entity_id in entity_ids {
            // エンティティのコンポーネントを取得
            let transform_option = world.get_component_mut::<Transform>(entity_id);
            let renderable_option = world.get_component_mut::<Renderable>(entity_id);
            
            // このエンティティに関連するアニメーションを処理
            let animations = animation_manager.get_animations_for_entity(entity_id);
            
            for &animation in animations {
                match &animation.animation_type {
                    AnimationType::Move { start_pos, end_pos, .. } => {
                        if let Some(transform) = transform_option {
                            transform.position = animation.calculate_value(*start_pos, *end_pos);
                        }
                    },
                    AnimationType::Rotate { start_angle, end_angle, .. } => {
                        if let Some(transform) = transform_option {
                            transform.rotation = animation.calculate_value(*start_angle, *end_angle);
                        }
                    },
                    AnimationType::Scale { start_scale, end_scale, .. } => {
                        if let Some(transform) = transform_option {
                            transform.scale = animation.calculate_value(*start_scale, *end_scale);
                        }
                    },
                    AnimationType::Fade { start_opacity, end_opacity, .. } => {
                        if let Some(renderable) = renderable_option {
                            renderable.opacity = animation.calculate_value(*start_opacity, *end_opacity);
                        }
                    },
                }
            }
        }
        
        Ok(())
    }
} 