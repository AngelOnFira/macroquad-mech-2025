use crate::WorldPos;
use std::collections::VecDeque;
use uuid::Uuid;

/// Generic object pool for managing reusable objects
pub struct ObjectPool<T> {
    available: VecDeque<T>,
    max_size: usize,
    create_fn: Box<dyn Fn() -> T + Send + Sync>,
    reset_fn: Box<dyn Fn(&mut T) + Send + Sync>,
}

impl<T> ObjectPool<T> {
    /// Create a new object pool
    pub fn new<F, R>(max_size: usize, create_fn: F, reset_fn: R) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
        R: Fn(&mut T) + Send + Sync + 'static,
    {
        let mut pool = Self {
            available: VecDeque::new(),
            max_size,
            create_fn: Box::new(create_fn),
            reset_fn: Box::new(reset_fn),
        };

        // Pre-populate the pool
        for _ in 0..max_size.min(10) {
            pool.available.push_back((pool.create_fn)());
        }

        pool
    }

    /// Get an object from the pool
    pub fn get(&mut self) -> T {
        if let Some(mut obj) = self.available.pop_front() {
            (self.reset_fn)(&mut obj);
            obj
        } else {
            (self.create_fn)()
        }
    }

    /// Return an object to the pool
    pub fn return_object(&mut self, obj: T) {
        if self.available.len() < self.max_size {
            self.available.push_back(obj);
        }
        // If pool is full, just drop the object
    }

    /// Get current pool size
    pub fn available_count(&self) -> usize {
        self.available.len()
    }

    /// Get maximum pool size
    pub fn max_size(&self) -> usize {
        self.max_size
    }
}

/// Pooled projectile for efficient memory management
#[derive(Debug, Clone)]
pub struct PooledProjectile {
    pub id: Uuid,
    pub position: WorldPos,
    pub velocity: (f32, f32),
    pub damage: u32,
    pub owner_mech_id: Uuid,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub active: bool,
}

impl Default for PooledProjectile {
    fn default() -> Self {
        Self::new()
    }
}

impl PooledProjectile {
    /// Create a new pooled projectile
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            position: WorldPos::new(0.0, 0.0),
            velocity: (0.0, 0.0),
            damage: 0,
            owner_mech_id: Uuid::nil(),
            lifetime: 0.0,
            max_lifetime: 5.0,
            active: false,
        }
    }

    /// Initialize projectile with specific values
    pub fn initialize(
        &mut self,
        position: WorldPos,
        velocity: (f32, f32),
        damage: u32,
        owner_mech_id: Uuid,
        max_lifetime: f32,
    ) {
        self.id = Uuid::new_v4();
        self.position = position;
        self.velocity = velocity;
        self.damage = damage;
        self.owner_mech_id = owner_mech_id;
        self.lifetime = 0.0;
        self.max_lifetime = max_lifetime;
        self.active = true;
    }

    /// Update projectile position and lifetime
    pub fn update(&mut self, delta_time: f32) -> bool {
        if !self.active {
            return false;
        }

        self.position.x += self.velocity.0 * delta_time;
        self.position.y += self.velocity.1 * delta_time;
        self.lifetime += delta_time;

        // Return false if projectile should be removed
        self.lifetime < self.max_lifetime
    }

    /// Reset projectile for reuse
    pub fn reset(&mut self) {
        self.active = false;
        self.lifetime = 0.0;
        self.damage = 0;
        self.velocity = (0.0, 0.0);
        self.position = WorldPos::new(0.0, 0.0);
        self.owner_mech_id = Uuid::nil();
    }

    /// Check if projectile is still active and valid
    pub fn is_active(&self) -> bool {
        self.active && self.lifetime < self.max_lifetime
    }
}

/// Pooled visual effect for efficient rendering
#[derive(Debug, Clone)]
pub struct PooledEffect {
    pub id: Uuid,
    pub effect_type: EffectType,
    pub position: WorldPos,
    pub duration: f32,
    pub max_duration: f32,
    pub intensity: f32,
    pub color: (f32, f32, f32, f32), // RGBA
    pub active: bool,
}

/// Types of visual effects
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EffectType {
    Explosion,
    LaserBeam,
    ShieldHit,
    Repair,
    Upgrade,
    Damage,
    Heal,
}

impl Default for PooledEffect {
    fn default() -> Self {
        Self::new()
    }
}

impl PooledEffect {
    /// Create a new pooled effect
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            effect_type: EffectType::Explosion,
            position: WorldPos::new(0.0, 0.0),
            duration: 0.0,
            max_duration: 1.0,
            intensity: 1.0,
            color: (1.0, 1.0, 1.0, 1.0),
            active: false,
        }
    }

    /// Initialize effect with specific values
    pub fn initialize(
        &mut self,
        effect_type: EffectType,
        position: WorldPos,
        max_duration: f32,
        intensity: f32,
        color: (f32, f32, f32, f32),
    ) {
        self.id = Uuid::new_v4();
        self.effect_type = effect_type;
        self.position = position;
        self.duration = 0.0;
        self.max_duration = max_duration;
        self.intensity = intensity;
        self.color = color;
        self.active = true;
    }

    /// Update effect duration
    pub fn update(&mut self, delta_time: f32) -> bool {
        if !self.active {
            return false;
        }

        self.duration += delta_time;

        // Return false if effect should be removed
        self.duration < self.max_duration
    }

    /// Reset effect for reuse
    pub fn reset(&mut self) {
        self.active = false;
        self.duration = 0.0;
        self.intensity = 1.0;
        self.color = (1.0, 1.0, 1.0, 1.0);
        self.position = WorldPos::new(0.0, 0.0);
    }

    /// Check if effect is still active and valid
    pub fn is_active(&self) -> bool {
        self.active && self.duration < self.max_duration
    }

    /// Get current alpha based on duration (fade out effect)
    pub fn get_alpha(&self) -> f32 {
        if self.max_duration <= 0.0 {
            return self.color.3;
        }

        let progress = self.duration / self.max_duration;
        let fade_alpha = (1.0 - progress).max(0.0);
        self.color.3 * fade_alpha
    }
}

/// Manager for all object pools
pub struct PoolManager {
    projectile_pool: ObjectPool<PooledProjectile>,
    effect_pool: ObjectPool<PooledEffect>,
}

impl PoolManager {
    /// Create a new pool manager
    pub fn new() -> Self {
        let projectile_pool = ObjectPool::new(
            200, // Max 200 projectiles
            PooledProjectile::new,
            |proj| proj.reset(),
        );

        let effect_pool = ObjectPool::new(
            500, // Max 500 effects
            PooledEffect::new,
            |effect| effect.reset(),
        );

        Self {
            projectile_pool,
            effect_pool,
        }
    }

    /// Get a projectile from the pool
    pub fn get_projectile(&mut self) -> PooledProjectile {
        self.projectile_pool.get()
    }

    /// Return a projectile to the pool
    pub fn return_projectile(&mut self, projectile: PooledProjectile) {
        self.projectile_pool.return_object(projectile);
    }

    /// Get an effect from the pool
    pub fn get_effect(&mut self) -> PooledEffect {
        self.effect_pool.get()
    }

    /// Return an effect to the pool
    pub fn return_effect(&mut self, effect: PooledEffect) {
        self.effect_pool.return_object(effect);
    }

    /// Get pool statistics
    pub fn get_stats(&self) -> PoolStats {
        PoolStats {
            projectiles_available: self.projectile_pool.available_count(),
            projectiles_max: self.projectile_pool.max_size(),
            effects_available: self.effect_pool.available_count(),
            effects_max: self.effect_pool.max_size(),
        }
    }
}

/// Statistics about pool usage
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub projectiles_available: usize,
    pub projectiles_max: usize,
    pub effects_available: usize,
    pub effects_max: usize,
}

impl Default for PoolManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_pool_basic() {
        let mut pool = ObjectPool::new(5, PooledProjectile::new, |proj| proj.reset());

        assert!(pool.available_count() > 0);

        let mut proj = pool.get();
        proj.initialize(
            WorldPos::new(10.0, 10.0),
            (1.0, 0.0),
            25,
            Uuid::new_v4(),
            5.0,
        );

        assert!(proj.is_active());
        pool.return_object(proj);
    }

    #[test]
    fn test_projectile_lifecycle() {
        let mut proj = PooledProjectile::new();
        assert!(!proj.is_active());

        proj.initialize(
            WorldPos::new(0.0, 0.0),
            (10.0, 0.0),
            50,
            Uuid::new_v4(),
            2.0,
        );

        assert!(proj.is_active());
        assert_eq!(proj.position.x, 0.0);

        // Update for 1 second
        assert!(proj.update(1.0));
        assert_eq!(proj.position.x, 10.0);

        // Update past lifetime
        assert!(!proj.update(2.0));
    }

    #[test]
    fn test_effect_fade() {
        let mut effect = PooledEffect::new();
        effect.initialize(
            EffectType::Explosion,
            WorldPos::new(0.0, 0.0),
            2.0,
            1.0,
            (1.0, 0.0, 0.0, 1.0),
        );

        // At start, alpha should be full
        assert!((effect.get_alpha() - 1.0).abs() < 0.01);

        // Update to halfway
        effect.update(1.0);
        let mid_alpha = effect.get_alpha();
        assert!(mid_alpha > 0.0 && mid_alpha < 1.0);

        // Update to end
        effect.update(1.0);
        assert!(effect.get_alpha() < 0.1);
    }

    #[test]
    fn test_pool_manager() {
        let mut manager = PoolManager::new();
        let stats = manager.get_stats();

        assert!(stats.projectiles_available > 0);
        assert!(stats.effects_available > 0);

        let mut proj = manager.get_projectile();
        proj.initialize(WorldPos::new(0.0, 0.0), (1.0, 1.0), 25, Uuid::new_v4(), 1.0);

        manager.return_projectile(proj);

        let mut effect = manager.get_effect();
        effect.initialize(
            EffectType::LaserBeam,
            WorldPos::new(5.0, 5.0),
            0.5,
            0.8,
            (0.0, 1.0, 0.0, 1.0),
        );

        manager.return_effect(effect);
    }
}
