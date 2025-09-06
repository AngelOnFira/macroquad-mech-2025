use crate::{WorldPos, TILE_SIZE};
use serde::{Deserialize, Serialize};

/// Axis-Aligned Bounding Box for collision detection
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AABB {
    pub min: WorldPos,
    pub max: WorldPos,
}

impl AABB {
    /// Create a new AABB from min and max corners
    pub fn new(min: WorldPos, max: WorldPos) -> Self {
        Self { min, max }
    }

    /// Create AABB centered at position with given half-extents
    pub fn from_center_and_half_extents(center: WorldPos, half_width: f32, half_height: f32) -> Self {
        Self {
            min: WorldPos::new(center.x - half_width, center.y - half_height),
            max: WorldPos::new(center.x + half_width, center.y + half_height),
        }
    }

    /// Create AABB for a player at the given position
    pub fn player_bounds(position: WorldPos) -> Self {
        let radius = crate::balance::PLAYER_COLLISION_RADIUS * TILE_SIZE;
        Self::from_center_and_half_extents(position, radius, radius)
    }

    /// Create AABB for a mech at the given position
    pub fn mech_bounds(position: WorldPos) -> Self {
        let half_size = (crate::MECH_SIZE_TILES as f32 * TILE_SIZE) / 2.0;
        Self {
            min: position,
            max: WorldPos::new(position.x + half_size * 2.0, position.y + half_size * 2.0),
        }
    }

    /// Check if this AABB intersects with another
    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }

    /// Check if a point is inside this AABB
    pub fn contains_point(&self, point: WorldPos) -> bool {
        point.x >= self.min.x && point.x <= self.max.x
            && point.y >= self.min.y && point.y <= self.max.y
    }

    /// Get the center point of this AABB
    pub fn center(&self) -> WorldPos {
        WorldPos::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
        )
    }

    /// Get width and height
    pub fn size(&self) -> (f32, f32) {
        (self.max.x - self.min.x, self.max.y - self.min.y)
    }

    /// Expand AABB by the given amount on all sides
    pub fn expand(&self, amount: f32) -> Self {
        Self {
            min: WorldPos::new(self.min.x - amount, self.min.y - amount),
            max: WorldPos::new(self.max.x + amount, self.max.y + amount),
        }
    }
}

/// Collision manifold containing information about a collision
#[derive(Debug, Clone)]
pub struct CollisionManifold {
    /// Penetration depth (how far the objects overlap)
    pub penetration_depth: f32,
    /// Collision normal pointing from object A to object B
    pub normal: (f32, f32),
    /// Contact point in world space
    pub contact_point: WorldPos,
}

impl CollisionManifold {
    /// Calculate collision manifold between two AABBs
    pub fn aabb_vs_aabb(a: &AABB, b: &AABB) -> Option<Self> {
        if !a.intersects(b) {
            return None;
        }

        // Calculate overlap on each axis
        let x_overlap = (a.max.x - b.min.x).min(b.max.x - a.min.x);
        let y_overlap = (a.max.y - b.min.y).min(b.max.y - a.min.y);

        let (penetration_depth, normal) = if x_overlap < y_overlap {
            // Separate along X axis
            let normal_x = if a.center().x < b.center().x { -1.0 } else { 1.0 };
            (x_overlap, (normal_x, 0.0))
        } else {
            // Separate along Y axis
            let normal_y = if a.center().y < b.center().y { -1.0 } else { 1.0 };
            (y_overlap, (0.0, normal_y))
        };

        let contact_point = WorldPos::new(
            (a.max.x.min(b.max.x) + a.min.x.max(b.min.x)) / 2.0,
            (a.max.y.min(b.max.y) + a.min.y.max(b.min.y)) / 2.0,
        );

        Some(CollisionManifold {
            penetration_depth,
            normal,
            contact_point,
        })
    }
}

/// Types of collision responses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionResponse {
    /// Block movement (solid collision)
    Block,
    /// Apply damage and potentially kill
    Damage,
    /// Push away from collision
    Push,
    /// Trigger an event but don't block movement
    Trigger,
    /// No response (for detection only)
    None,
}

/// Collision layer for filtering which objects can collide
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionLayer {
    Player,
    Mech,
    Projectile,
    Resource,
    World, // Static world geometry
}

/// Collision filter for determining which layers can interact
#[derive(Debug, Clone, Copy)]
pub struct CollisionFilter {
    layers: u32, // Bitfield for layers this object is on
    mask: u32,   // Bitfield for layers this object can collide with
}

impl CollisionFilter {
    /// Create a new collision filter
    pub fn new(layers: &[CollisionLayer], mask: &[CollisionLayer]) -> Self {
        let layers = layers.iter().fold(0, |acc, &layer| acc | (1 << layer as u8));
        let mask = mask.iter().fold(0, |acc, &layer| acc | (1 << layer as u8));
        Self { layers, mask }
    }

    /// Check if this filter can collide with another
    pub fn can_collide_with(&self, other: &Self) -> bool {
        (self.mask & other.layers) != 0 && (other.mask & self.layers) != 0
    }

    /// Player collision filter - can collide with mechs and world
    pub fn player() -> Self {
        Self::new(
            &[CollisionLayer::Player],
            &[CollisionLayer::Mech, CollisionLayer::World],
        )
    }

    /// Mech collision filter - can collide with players and other mechs
    pub fn mech() -> Self {
        Self::new(
            &[CollisionLayer::Mech],
            &[CollisionLayer::Player, CollisionLayer::Mech],
        )
    }

    /// Projectile collision filter - can collide with mechs and world
    pub fn projectile() -> Self {
        Self::new(
            &[CollisionLayer::Projectile],
            &[CollisionLayer::Mech, CollisionLayer::World],
        )
    }
}

/// Collision shape for an entity
#[derive(Debug, Clone)]
pub struct CollisionShape {
    pub aabb: AABB,
    pub layer: CollisionLayer,
    pub response: CollisionResponse,
    pub filter: CollisionFilter,
}

impl CollisionShape {
    /// Create a player collision shape
    pub fn player(position: WorldPos) -> Self {
        Self {
            aabb: AABB::player_bounds(position),
            layer: CollisionLayer::Player,
            response: CollisionResponse::Block,
            filter: CollisionFilter::player(),
        }
    }

    /// Create a mech collision shape
    pub fn mech(position: WorldPos) -> Self {
        Self {
            aabb: AABB::mech_bounds(position),
            layer: CollisionLayer::Mech,
            response: CollisionResponse::Block,
            filter: CollisionFilter::mech(),
        }
    }

    /// Update the position of this collision shape
    pub fn update_position(&mut self, new_position: WorldPos) {
        let size = self.aabb.size();
        match self.layer {
            CollisionLayer::Player => {
                let radius = crate::balance::PLAYER_COLLISION_RADIUS * TILE_SIZE;
                self.aabb = AABB::from_center_and_half_extents(new_position, radius, radius);
            }
            CollisionLayer::Mech => {
                self.aabb = AABB::mech_bounds(new_position);
            }
            _ => {
                // For other types, maintain the same size
                self.aabb = AABB {
                    min: new_position,
                    max: WorldPos::new(new_position.x + size.0, new_position.y + size.1),
                };
            }
        }
    }
}

/// Collision query result
#[derive(Debug, Clone)]
pub struct CollisionQueryResult {
    pub manifold: CollisionManifold,
    pub response: CollisionResponse,
    pub other_layer: CollisionLayer,
}

/// Utilities for collision detection and response
pub struct CollisionUtils;

impl CollisionUtils {
    /// Check if movement from start to end position would collide with obstacles
    /// Returns the maximum safe movement vector (may be shorter than requested)
    pub fn calculate_safe_movement(
        start_pos: WorldPos,
        desired_movement: (f32, f32),
        collision_shape: &CollisionShape,
        obstacles: &[CollisionShape],
    ) -> (f32, f32) {
        let mut safe_movement = desired_movement;
        let end_pos = WorldPos::new(
            start_pos.x + desired_movement.0,
            start_pos.y + desired_movement.1,
        );

        let mut test_shape = collision_shape.clone();
        test_shape.update_position(end_pos);

        // Check collision with each obstacle
        for obstacle in obstacles {
            if !test_shape.filter.can_collide_with(&obstacle.filter) {
                continue;
            }

            if let Some(manifold) = CollisionManifold::aabb_vs_aabb(&test_shape.aabb, &obstacle.aabb) {
                // Adjust movement to avoid collision
                safe_movement.0 -= manifold.normal.0 * manifold.penetration_depth;
                safe_movement.1 -= manifold.normal.1 * manifold.penetration_depth;

                // Update test position with adjusted movement
                let adjusted_pos = WorldPos::new(
                    start_pos.x + safe_movement.0,
                    start_pos.y + safe_movement.1,
                );
                test_shape.update_position(adjusted_pos);
            }
        }

        safe_movement
    }

    /// Calculate separation vector to resolve overlap between two collision shapes
    pub fn calculate_separation(a: &CollisionShape, b: &CollisionShape) -> Option<(f32, f32)> {
        if let Some(manifold) = CollisionManifold::aabb_vs_aabb(&a.aabb, &b.aabb) {
            Some((
                manifold.normal.0 * manifold.penetration_depth,
                manifold.normal.1 * manifold.penetration_depth,
            ))
        } else {
            None
        }
    }

    /// Check if a mech is moving toward a player with sufficient velocity to cause damage
    pub fn should_cause_run_over_damage(
        mech_velocity: (f32, f32),
        mech_pos: WorldPos,
        player_pos: WorldPos,
        min_velocity_threshold: f32,
    ) -> bool {
        let velocity_magnitude = (mech_velocity.0 * mech_velocity.0 + mech_velocity.1 * mech_velocity.1).sqrt();
        
        if velocity_magnitude < min_velocity_threshold {
            return false;
        }

        // Check if mech is moving toward player
        let to_player = (player_pos.x - mech_pos.x, player_pos.y - mech_pos.y);
        let to_player_mag = (to_player.0 * to_player.0 + to_player.1 * to_player.1).sqrt();
        
        if to_player_mag == 0.0 {
            return false;
        }

        let to_player_normalized = (to_player.0 / to_player_mag, to_player.1 / to_player_mag);
        let velocity_normalized = (mech_velocity.0 / velocity_magnitude, mech_velocity.1 / velocity_magnitude);

        // Dot product to check if moving toward player (> 0.5 means within ~60 degrees)
        let dot_product = velocity_normalized.0 * to_player_normalized.0 + velocity_normalized.1 * to_player_normalized.1;
        dot_product > 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_intersection() {
        let a = AABB::new(WorldPos::new(0.0, 0.0), WorldPos::new(10.0, 10.0));
        let b = AABB::new(WorldPos::new(5.0, 5.0), WorldPos::new(15.0, 15.0));
        let c = AABB::new(WorldPos::new(20.0, 20.0), WorldPos::new(30.0, 30.0));

        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
        assert!(!a.intersects(&c));
        assert!(!c.intersects(&a));
    }

    #[test]
    fn test_collision_manifold() {
        let a = AABB::new(WorldPos::new(0.0, 0.0), WorldPos::new(10.0, 10.0));
        let b = AABB::new(WorldPos::new(5.0, 5.0), WorldPos::new(15.0, 15.0));

        let manifold = CollisionManifold::aabb_vs_aabb(&a, &b).unwrap();
        assert!(manifold.penetration_depth > 0.0);
        assert!(manifold.penetration_depth <= 5.0);
    }

    #[test]
    fn test_run_over_detection() {
        let mech_pos = WorldPos::new(0.0, 0.0);
        let player_pos = WorldPos::new(10.0, 0.0);
        let mech_velocity = (2.0, 0.0); // Moving toward player
        
        assert!(CollisionUtils::should_cause_run_over_damage(
            mech_velocity, 
            mech_pos, 
            player_pos, 
            1.0
        ));

        // Moving away from player
        let mech_velocity_away = (-2.0, 0.0);
        assert!(!CollisionUtils::should_cause_run_over_damage(
            mech_velocity_away, 
            mech_pos, 
            player_pos, 
            1.0
        ));

        // Moving too slowly
        let slow_velocity = (0.1, 0.0);
        assert!(!CollisionUtils::should_cause_run_over_damage(
            slow_velocity, 
            mech_pos, 
            player_pos, 
            1.0
        ));
    }
}