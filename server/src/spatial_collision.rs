use crate::game::Game;
use shared::*;
use uuid::Uuid;

/// Data types for spatial entities
#[derive(Debug, Clone)]
pub enum SpatialEntityData {
    Player(Uuid),
    Mech(Uuid),
    Resource(ResourceType),
    Projectile(ProjectileData),
}

#[derive(Debug, Clone)]
pub struct ProjectileData {
    pub damage: u32,
    pub owner_mech_id: Uuid,
    pub velocity: (f32, f32),
}

/// Spatial collision manager for the game
pub struct SpatialCollisionManager {
    player_grid: SpatialGrid<SpatialEntityData>,
    mech_grid: SpatialGrid<SpatialEntityData>,
    resource_grid: SpatialGrid<SpatialEntityData>,
    projectile_grid: SpatialGrid<SpatialEntityData>,
}

impl SpatialCollisionManager {
    /// Create a new spatial collision manager
    pub fn new() -> Self {
        // Use different cell sizes for different entity types
        let player_cell_size = TILE_SIZE * 2.0; // Players are small and fast
        let mech_cell_size = TILE_SIZE * 4.0; // Mechs are large
        let resource_cell_size = TILE_SIZE * 3.0; // Resources are medium
        let projectile_cell_size = TILE_SIZE * 1.5; // Projectiles are small but fast

        Self {
            player_grid: SpatialGrid::for_arena(player_cell_size),
            mech_grid: SpatialGrid::for_arena(mech_cell_size),
            resource_grid: SpatialGrid::for_arena(resource_cell_size),
            projectile_grid: SpatialGrid::for_arena(projectile_cell_size),
        }
    }

    /// Clear all spatial grids
    pub fn clear(&mut self) {
        self.player_grid.clear();
        self.mech_grid.clear();
        self.resource_grid.clear();
        self.projectile_grid.clear();
    }

    /// Add a player to the spatial collision system
    pub fn add_player(&mut self, player_id: Uuid, position: WorldPos) {
        let entity = SpatialEntity::new(
            player_id,
            position,
            PLAYER_COLLISION_RADIUS,
            SpatialEntityData::Player(player_id),
        );
        self.player_grid.insert(entity);
    }

    /// Add a mech to the spatial collision system
    pub fn add_mech(&mut self, mech_id: Uuid, position: WorldPos) {
        let entity = SpatialEntity::new(
            mech_id,
            position,
            MECH_COLLISION_RADIUS,
            SpatialEntityData::Mech(mech_id),
        );
        self.mech_grid.insert(entity);
    }

    /// Add a resource to the spatial collision system
    pub fn add_resource(&mut self, resource_id: Uuid, position: WorldPos) {
        let entity = SpatialEntity::new(
            resource_id,
            position,
            RESOURCE_COLLISION_RADIUS,
            SpatialEntityData::Resource(ResourceType::ScrapMetal),
        );
        self.resource_grid.insert(entity);
    }

    /// Add a projectile to the spatial collision system
    pub fn add_projectile(&mut self, projectile_id: Uuid, position: WorldPos) {
        let entity = SpatialEntity::new(
            projectile_id,
            position,
            PROJECTILE_COLLISION_RADIUS,
            SpatialEntityData::Projectile(ProjectileData {
                damage: 0,
                owner_mech_id: Uuid::nil(),
                velocity: (0.0, 0.0),
            }),
        );
        self.projectile_grid.insert(entity);
    }

    /// Update all spatial grids with current game state
    pub fn update(&mut self, game: &Game) {
        // Clear all grids
        self.clear();

        // Update player positions
        for player in game.players.values() {
            if let PlayerLocation::OutsideWorld(pos) = player.location {
                let entity = SpatialEntity::new(
                    player.id,
                    pos,
                    PLAYER_COLLISION_RADIUS,
                    SpatialEntityData::Player(player.id),
                );
                self.player_grid.insert(entity);
            }
        }

        // Update mech positions
        for mech in game.mechs.values() {
            let entity = SpatialEntity::new(
                mech.id,
                mech.world_position,
                MECH_COLLISION_RADIUS,
                SpatialEntityData::Mech(mech.id),
            );
            self.mech_grid.insert(entity);
        }

        // Update resource positions
        for resource in &game.get_resources() {
            let entity = SpatialEntity::new(
                resource.id,
                resource.position.to_world_pos(),
                RESOURCE_COLLISION_RADIUS,
                SpatialEntityData::Resource(resource.resource_type),
            );
            self.resource_grid.insert(entity);
        }

        // Update projectile positions
        for projectile in game.projectiles.values() {
            let entity = SpatialEntity::new(
                projectile.id,
                projectile.position,
                PROJECTILE_COLLISION_RADIUS,
                SpatialEntityData::Projectile(ProjectileData {
                    damage: projectile.damage,
                    owner_mech_id: projectile.owner_mech_id,
                    velocity: projectile.velocity,
                }),
            );
            self.projectile_grid.insert(entity);
        }
    }

    /// Check for player-resource collisions
    pub fn check_player_resource_collisions(
        &self,
        player_id: Uuid,
        player_pos: WorldPos,
    ) -> Vec<Uuid> {
        let query_results = self
            .resource_grid
            .query_radius(player_pos, RESOURCE_PICKUP_DISTANCE);

        query_results
            .into_iter()
            .filter_map(|result| {
                if result.distance <= RESOURCE_PICKUP_DISTANCE {
                    Some(result.entity.id)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Check for player-mech collisions
    pub fn check_player_mech_collisions(
        &self,
        player_pos: WorldPos,
        player_team: TeamId,
    ) -> Vec<Uuid> {
        let query_results = self
            .mech_grid
            .query_radius(player_pos, MECH_COLLISION_DISTANCE);

        query_results
            .into_iter()
            .filter_map(|result| {
                if result.distance <= MECH_COLLISION_DISTANCE {
                    Some(result.entity.id)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Check for projectile-mech collisions
    pub fn check_projectile_mech_collisions(&self, game: &Game) -> Vec<(Uuid, Uuid)> {
        let mut collisions = Vec::new();

        for projectile in game.projectiles.values() {
            let query_results = self.mech_grid.query_radius(
                projectile.position,
                PROJECTILE_COLLISION_RADIUS + MECH_COLLISION_RADIUS,
            );

            for result in query_results {
                if let SpatialEntityData::Mech(mech_id) = result.entity.data {
                    // Don't collide with owner mech
                    if mech_id != projectile.owner_mech_id {
                        // Check if projectile actually hits the mech
                        if result.distance <= PROJECTILE_COLLISION_RADIUS + MECH_COLLISION_RADIUS {
                            collisions.push((projectile.id, mech_id));
                        }
                    }
                }
            }
        }

        collisions
    }

    /// Find nearest enemy mech for weapon targeting
    pub fn find_nearest_enemy_mech(
        &self,
        mech_pos: WorldPos,
        mech_team: TeamId,
        game: &Game,
    ) -> Option<Uuid> {
        let query_results = self.mech_grid.query_radius(mech_pos, WEAPON_MAX_RANGE);

        query_results
            .into_iter()
            .filter_map(|result| {
                if let SpatialEntityData::Mech(mech_id) = result.entity.data {
                    if let Some(mech) = game.mechs.get(&mech_id) {
                        if mech.team != mech_team {
                            return Some((mech_id, result.distance));
                        }
                    }
                }
                None
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(mech_id, _)| mech_id)
    }

    /// Check for overlapping entities (for debugging)
    pub fn check_overlapping_entities(
        &self,
        pos: WorldPos,
        radius: f32,
    ) -> Vec<SpatialEntity<SpatialEntityData>> {
        let mut overlapping = Vec::new();

        // Check all grids
        overlapping.extend(
            self.player_grid
                .query_radius(pos, radius)
                .into_iter()
                .map(|r| r.entity),
        );
        overlapping.extend(
            self.mech_grid
                .query_radius(pos, radius)
                .into_iter()
                .map(|r| r.entity),
        );
        overlapping.extend(
            self.resource_grid
                .query_radius(pos, radius)
                .into_iter()
                .map(|r| r.entity),
        );
        overlapping.extend(
            self.projectile_grid
                .query_radius(pos, radius)
                .into_iter()
                .map(|r| r.entity),
        );

        overlapping
    }

    /// Get debug information about all spatial grids
    pub fn get_debug_info(&self) -> SpatialCollisionDebugInfo {
        SpatialCollisionDebugInfo {
            player_grid: self.player_grid.debug_info(),
            mech_grid: self.mech_grid.debug_info(),
            resource_grid: self.resource_grid.debug_info(),
            projectile_grid: self.projectile_grid.debug_info(),
        }
    }
}

/// Debug information for spatial collision system
#[derive(Debug)]
pub struct SpatialCollisionDebugInfo {
    pub player_grid: SpatialGridDebugInfo,
    pub mech_grid: SpatialGridDebugInfo,
    pub resource_grid: SpatialGridDebugInfo,
    pub projectile_grid: SpatialGridDebugInfo,
}

// Add collision radius constants to balance.rs if they don't exist
impl Default for SpatialCollisionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Game;

    #[test]
    fn test_spatial_collision_manager_creation() {
        let manager = SpatialCollisionManager::new();
        let debug_info = manager.get_debug_info();

        assert!(debug_info.player_grid.total_cells > 0);
        assert!(debug_info.mech_grid.total_cells > 0);
        assert!(debug_info.resource_grid.total_cells > 0);
        assert!(debug_info.projectile_grid.total_cells > 0);
    }

    #[test]
    fn test_spatial_collision_update() {
        let mut manager = SpatialCollisionManager::new();
        let game = Game::new();

        manager.update(&game);

        let debug_info = manager.get_debug_info();
        assert_eq!(debug_info.player_grid.total_entities, 0);
        assert_eq!(debug_info.mech_grid.total_entities, 0);
        assert_eq!(debug_info.resource_grid.total_entities, 0);
        assert_eq!(debug_info.projectile_grid.total_entities, 0);
    }
}
