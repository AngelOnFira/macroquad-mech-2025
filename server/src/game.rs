use std::collections::HashMap;
use tokio::sync::broadcast;
use uuid::Uuid;

use shared::*;
use shared::tile_entity::{TileMap, TileContent, StaticTile, TransitionType, TileVisual, Material};
use shared::vision::VisionSystem;
use shared::components::{Position, Station};
use shared::mech_layout::MechLayoutGenerator;
use shared::stations::StationRegistry;
use shared::object_pool::PoolManager;
use crate::spatial_collision::SpatialCollisionManager;
use crate::systems::SystemManager;
use crate::entity_storage::EntityStorage;

pub struct Game {
    pub players: HashMap<Uuid, Player>,
    pub mechs: HashMap<Uuid, Mech>,
    // Resources are now stored in entity_storage with ResourcePickup component
    pub projectiles: HashMap<Uuid, PooledProjectile>,
    pub active_effects: HashMap<Uuid, PooledEffect>,
    pub tick_count: u64,
    pub spatial_collision: SpatialCollisionManager,
    pub station_registry: StationRegistry,
    pub pool_manager: PoolManager,
    pub system_manager: SystemManager,
    pub tile_map: TileMap,
    pub entity_storage: EntityStorage,
    pub vision_system: VisionSystem,
}

pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub team: TeamId,
    pub location: PlayerLocation,
    pub carrying_resource: Option<ResourceType>,
    pub operating_station: Option<Uuid>,
}

pub struct Mech {
    pub id: Uuid,
    pub team: TeamId,
    pub position: TilePos,
    pub health: u32,
    pub max_health: u32,
    pub shield: u32,
    pub max_shield: u32,
    pub upgrades: MechUpgrades,
    pub stations: HashMap<Uuid, StationInstance>,
    pub interior: MechInterior,
    pub resource_inventory: HashMap<ResourceType, u32>,
    pub velocity: (f32, f32), // tiles per second
    pub world_position: WorldPos, // For smooth movement
}

impl Clone for Player {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            team: self.team,
            location: self.location,
            carrying_resource: self.carrying_resource,
            operating_station: self.operating_station,
        }
    }
}

pub struct Resource {
    pub id: Uuid,
    pub position: TilePos,
    pub resource_type: ResourceType,
}

// Projectile is now handled by PooledProjectile from the object_pool module

impl Game {
    /// Get all resources from entity storage
    pub fn get_resources(&self) -> Vec<Resource> {
        let mut resources = Vec::new();
        for (entity_id, pickup) in &self.entity_storage.resource_pickups {
            if let Some(pos) = self.entity_storage.positions.get(entity_id) {
                resources.push(Resource {
                    id: *entity_id,
                    position: pos.tile,
                    resource_type: pickup.resource_type,
                });
            }
        }
        resources
    }
    
    /// Get resource by ID
    pub fn get_resource(&self, id: Uuid) -> Option<Resource> {
        if let Some(pickup) = self.entity_storage.resource_pickups.get(&id) {
            if let Some(pos) = self.entity_storage.positions.get(&id) {
                return Some(Resource {
                    id,
                    position: pos.tile,
                    resource_type: pickup.resource_type,
                });
            }
        }
        None
    }
    
    /// Remove a resource entity
    pub fn remove_resource(&mut self, id: Uuid) {
        self.entity_storage.destroy_entity(id);
        self.tile_map.remove_tile(
            self.entity_storage.positions.get(&id)
                .map(|p| p.tile)
                .unwrap_or(TilePos::new(0, 0))
        );
    }
    
    pub fn new() -> Self {
        // Initialize the hybrid tile map
        let mut tile_map = TileMap::new();
        
        // Initialize world with grass tiles
        for x in 0..ARENA_WIDTH_TILES {
            for y in 0..ARENA_HEIGHT_TILES {
                tile_map.set_world_tile(
                    TilePos::new(x as i32, y as i32),
                    TileContent::Static(StaticTile::Grass)
                );
            }
        }
        
        let mut game = Self {
            players: HashMap::new(),
            mechs: HashMap::new(),
            projectiles: HashMap::new(),
            active_effects: HashMap::new(),
            tick_count: 0,
            spatial_collision: SpatialCollisionManager::new(),
            station_registry: StationRegistry::new(),
            pool_manager: PoolManager::new(),
            system_manager: SystemManager::new(),
            tile_map,
            entity_storage: EntityStorage::new(),
            vision_system: VisionSystem::new(),
        };
        
        // Initialize mechs and update tiles
        game.create_initial_mechs();
        
        game
    }
    
    /// Add an AI player to the game
    pub fn add_ai_player(&mut self, difficulty: f32, personality: Option<ai::Personality>) -> Option<Uuid> {
        // Count teams for balancing
        let red_count = self.players.values().filter(|p| p.team == TeamId::Red).count();
        let blue_count = self.players.values().filter(|p| p.team == TeamId::Blue).count();
        
        // Get the AI system from the system manager
        let mut system_manager = std::mem::take(&mut self.system_manager);
        let result = if let Some(ai_system) = system_manager.get_system_mut::<crate::systems::ai::AISystem>() {
            // Add the AI player
            let (ai_id, player) = ai_system.add_ai_player(difficulty, personality, red_count, blue_count);
            self.players.insert(ai_id, player);
            Some(ai_id)
        } else {
            log::error!("AI system not found in system manager");
            None
        };
        self.system_manager = system_manager;
        result
    }
    
    /// Remove an AI player from the game
    pub fn remove_ai_player(&mut self, ai_id: Uuid) {
        // Remove from players
        self.players.remove(&ai_id);
        
        // Get the AI system from the system manager
        let mut system_manager = std::mem::take(&mut self.system_manager);
        if let Some(ai_system) = system_manager.get_system_mut::<crate::systems::ai::AISystem>() {
            // Remove the AI player
            ai_system.remove_ai_player(ai_id);
        } else {
            log::error!("AI system not found in system manager");
        }
        self.system_manager = system_manager;
    }
    
    /// Get list of all AI players
    pub fn get_ai_players(&self) -> Vec<Uuid> {
        self.players.iter()
            .filter(|(_, p)| p.name.starts_with("AI_"))
            .map(|(id, _)| *id)
            .collect()
    }

    pub fn create_initial_mechs(&mut self) {
        // Red team mech
        let red_mech_pos = TilePos::new(RED_MECH_SPAWN.0, RED_MECH_SPAWN.1);
        let red_mech = self.create_mech(red_mech_pos, TeamId::Red);
        let red_mech_id = red_mech.id;
        self.mechs.insert(red_mech.id, red_mech);

        // Blue team mech
        let blue_mech_pos = TilePos::new(BLUE_MECH_SPAWN.0, BLUE_MECH_SPAWN.1);
        let blue_mech = self.create_mech(blue_mech_pos, TeamId::Blue);
        let blue_mech_id = blue_mech.id;
        self.mechs.insert(blue_mech.id, blue_mech);
        
        // Update tiles for both mechs
        self.update_mech_tiles(red_mech_id, red_mech_pos);
        self.update_mech_tiles(blue_mech_id, blue_mech_pos);
    }
    
    pub fn update_player_visibility(&mut self, tx: &broadcast::Sender<(Uuid, ServerMessage)>) {
        // Skip visibility updates every few ticks to reduce network traffic
        if self.tick_count % 5 != 0 {
            return;
        }
        
        // Calculate visibility for each player
        for (player_id, player) in &self.players {
            let world_pos = match player.location {
                PlayerLocation::OutsideWorld(pos) => pos,
                PlayerLocation::InsideMech { pos, .. } => pos,
            };
            
            // Calculate visibility using the vision system
            let visibility = self.vision_system.calculate_visibility(
                *player_id,
                world_pos,
                100.0, // Base vision range
                &self.tile_map,
                &self.entity_storage,
            );
            
            // Convert visible tiles to visuals
            let mut visible_tiles = Vec::new();
            for tile_pos in &visibility.visible_tiles {
                if let Some(tile_content) = self.tile_map.get_world_tile(*tile_pos) {
                    let visual = match tile_content {
                        TileContent::Empty => continue,
                        TileContent::Static(static_tile) => {
                            // Convert static tile to visual
                            match static_tile {
                                StaticTile::Grass => TileVisual::Floor {
                                    material: Material::Metal,
                                    wear: 0,
                                },
                                StaticTile::Rock => TileVisual::Wall {
                                    material: Material::Reinforced,
                                },
                                StaticTile::MetalFloor => TileVisual::Floor {
                                    material: Material::Metal,
                                    wear: 0,
                                },
                                StaticTile::CargoFloor { wear } => TileVisual::Floor {
                                    material: Material::Metal,
                                    wear,
                                },
                                StaticTile::MetalWall => TileVisual::Wall {
                                    material: Material::Metal,
                                },
                                StaticTile::ReinforcedWall => TileVisual::Wall {
                                    material: Material::Reinforced,
                                },
                                StaticTile::Window { facing } => TileVisual::Window {
                                    broken: false,
                                    facing,
                                },
                                StaticTile::ReinforcedWindow { facing, .. } => TileVisual::Window {
                                    broken: false,
                                    facing,
                                },
                                StaticTile::TransitionZone { .. } => TileVisual::TransitionFade {
                                    progress: 0.0,
                                },
                                _ => continue,
                            }
                        }
                        TileContent::Entity(entity_id) => {
                            // Get entity visual
                            if let Some(station) = self.entity_storage.stations.get(&entity_id) {
                                TileVisual::Station {
                                    station_type: station.station_type,
                                    active: station.operating,
                                }
                            } else {
                                continue;
                            }
                        }
                    };
                    
                    visible_tiles.push((*tile_pos, visual));
                }
            }
            
            // Send visibility update to player
            let _ = tx.send((*player_id, ServerMessage::VisibilityUpdate {
                visible_tiles,
                player_position: world_pos,
            }));
        }
    }

    fn create_mech(&mut self, position: TilePos, team: TeamId) -> Mech {
        let id = Uuid::new_v4();
        let mut mech_stations = HashMap::new();
        let interior = MechLayoutGenerator::create_mech_interior(&mut mech_stations);

        // Convert MechStations to Stations using the registry
        let mut stations = HashMap::new();
        for (station_id, mech_station) in mech_stations {
            let station = self.station_registry.create_station(
                mech_station.station_type,
                mech_station.floor,
                mech_station.position,
            ).expect("Failed to create station from registry");
            stations.insert(station_id, station);
        }

        Mech {
            id,
            team,
            position,
            health: MECH_INITIAL_HEALTH,
            max_health: MECH_MAX_HEALTH,
            shield: MECH_INITIAL_SHIELD,
            max_shield: MECH_MAX_SHIELD,
            upgrades: MechUpgrades {
                laser_level: INITIAL_UPGRADE_LEVEL,
                projectile_level: INITIAL_UPGRADE_LEVEL,
                engine_level: INITIAL_UPGRADE_LEVEL,
                shield_level: INITIAL_UPGRADE_LEVEL,
            },
            stations,
            interior,
            resource_inventory: HashMap::new(),
            velocity: (0.0, 0.0),
            world_position: position.to_world_pos(),
        }
    }
    
    fn update_mech_tiles(&mut self, mech_id: Uuid, mech_pos: TilePos) {
        // Create the mech tile map for this mech
        let mech_tile_map = self.tile_map.create_mech(mech_id, mech_pos);
        mech_tile_map.position = mech_pos;
        
        // Populate the mech interior from the layout
        if let Some(mech) = self.mechs.get(&mech_id) {
            for (floor_idx, floor_layout) in mech.interior.floors.iter().enumerate() {
                if let Some(floor_map) = mech_tile_map.floors.get_mut(floor_idx) {
                    // Add tiles from the layout
                    for (y, row) in floor_layout.tiles.iter().enumerate() {
                        for (x, tile_content) in row.iter().enumerate() {
                            let tile_pos = TilePos::new(x as i32, y as i32);
                            
                            match tile_content {
                                TileContent::Static(static_tile) => {
                                    floor_map.static_tiles.insert(tile_pos, *static_tile);
                                }
                                TileContent::Entity(id) => {
                                    floor_map.entity_tiles.insert(tile_pos, *id);
                                }
                                TileContent::Empty => {}
                            }
                        }
                    }
                    
                    // Add station entities
                    for (station_id, station) in &mech.stations {
                        if station.floor == floor_idx as u8 {
                            floor_map.entity_tiles.insert(station.position, *station_id);
                            
                            // Add to entity storage
                            self.entity_storage.add_entity(
                                *station_id,
                                Position {
                                    tile: station.position,
                                    world: WorldPos::from_tile(station.position),
                                    floor: Some(floor_idx as u8),
                                    mech_id: Some(mech_id),
                                }
                            );
                            
                            // Add station component
                            self.entity_storage.add_station(
                                *station_id,
                                Station {
                                    station_type: station.station_type,
                                    interaction_range: 1.5,
                                    power_required: 10.0,
                                    operating: false,
                                }
                            );
                        }
                    }
                }
            }
        }
        
        // Add door tiles at the bottom center of the mech - 2 blocks wide
        let door_x1 = mech_pos.x + (MECH_SIZE_TILES / 2) - 1;
        let door_x2 = mech_pos.x + (MECH_SIZE_TILES / 2);
        let door_y = mech_pos.y + MECH_SIZE_TILES - 1;
        
        self.tile_map.set_world_tile(
            TilePos::new(door_x1, door_y),
            TileContent::Static(StaticTile::TransitionZone {
                zone_id: 0,
                transition_type: TransitionType::MechEntrance { stage: 0 },
            })
        );
        self.tile_map.set_world_tile(
            TilePos::new(door_x2, door_y),
            TileContent::Static(StaticTile::TransitionZone {
                zone_id: 1,
                transition_type: TransitionType::MechEntrance { stage: 1 },
            })
        );
        
        // Add resource drop-off zone on top of the mech (roof area)
        let dropoff_x = mech_pos.x + (MECH_SIZE_TILES / 2) - 1;
        let dropoff_y = mech_pos.y;
        
        // For now, just mark the area with metal floor tiles
        // TODO: Add proper resource dropoff entity when needed
        for dy in 0..3 {
            for dx in 0..3 {
                self.tile_map.set_world_tile(
                    TilePos::new(dropoff_x + dx, dropoff_y + dy),
                    TileContent::Static(StaticTile::CargoFloor { wear: 0 })
                );
            }
        }
    }
    
    pub fn spawn_resource_with_behavior(&mut self, position: TilePos, resource_type: ResourceType) -> Uuid {
        use shared::components::*;
        
        // Create the entity
        let entity_id = self.entity_storage.create_entity(format!("Resource_{:?}", resource_type));
        
        // Add position
        self.entity_storage.add_position(entity_id, Position {
            tile: position,
            world: position.to_world_pos(),
            floor: None,
            mech_id: None,
        });
        
        // Add resource pickup behavior
        self.entity_storage.resource_pickups.insert(entity_id, ResourcePickup {
            resource_type,
            auto_pickup: true,
            pickup_range: 24.0, // 1.5 tiles
            respawn_time: Some(30.0), // Respawn after 30 seconds
        });
        
        // Add proximity trigger for visual feedback
        self.entity_storage.proximity_triggers.insert(entity_id, ProximityTrigger {
            range: 32.0, // 2 tiles
            trigger_for_teams: None, // All teams
            cooldown: 0.5,
            last_triggered: HashMap::new(),
        });
        
        // Add to tile map
        self.tile_map.set_entity_tile(position, entity_id);
        
        entity_id
    }
    
    pub fn spawn_mech_entrance(&mut self, position: TilePos, mech_id: Uuid, team: TeamId) -> Uuid {
        use shared::components::*;
        
        // Create the entity
        let entity_id = self.entity_storage.create_entity("MechEntrance".to_string());
        
        // Add position
        self.entity_storage.add_position(entity_id, Position {
            tile: position,
            world: position.to_world_pos(),
            floor: None,
            mech_id: None,
        });
        
        // Add mech entrance behavior
        self.entity_storage.mech_entrances.insert(entity_id, MechEntrance {
            mech_id,
            target_floor: 0,
            entry_position: WorldPos::new(24.0, 40.0), // Center of first floor
            team_restricted: Some(team),
        });
        
        // Add proximity trigger for UI prompt
        self.entity_storage.proximity_triggers.insert(entity_id, ProximityTrigger {
            range: 16.0, // 1 tile
            trigger_for_teams: Some(vec![team]),
            cooldown: 0.1,
            last_triggered: HashMap::new(),
        });
        
        // Add to tile map
        self.tile_map.set_entity_tile(position, entity_id);
        
        entity_id
    }
    
    pub fn spawn_resource_dropoff(&mut self, position: TilePos, mech_id: Uuid, team: TeamId) -> Uuid {
        use shared::components::*;
        
        // Create the entity
        let entity_id = self.entity_storage.create_entity("ResourceDropoff".to_string());
        
        // Add position
        self.entity_storage.add_position(entity_id, Position {
            tile: position,
            world: position.to_world_pos(),
            floor: None,
            mech_id: None,
        });
        
        // Add auto-interact for resource dropping
        self.entity_storage.auto_interacts.insert(entity_id, AutoInteract {
            interaction_type: AutoInteractionType::DropResource,
            range: 16.0, // 1 tile
            conditions: vec![
                InteractionCondition::PlayerOnTeam(team),
                InteractionCondition::PlayerCarrying(ResourceType::ScrapMetal), // Example - could be any
            ],
        });
        
        // Add to tile map
        self.tile_map.set_entity_tile(position, entity_id);
        
        entity_id
    }

    pub fn spawn_initial_resources(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Spawn 5-8 initial resources randomly
        let num_initial_resources = rng.gen_range(5..=8);
        let resource_types = [
            ResourceType::ScrapMetal,
            ResourceType::ComputerComponents,
            ResourceType::Wiring,
            ResourceType::Batteries,
        ];
        
        for _ in 0..num_initial_resources {
            // Try to find a valid spawn position
            let mut attempts = 0;
            const MAX_ATTEMPTS: i32 = 50;
            
            while attempts < MAX_ATTEMPTS {
                // Generate random position (avoiding edges)
                let x = rng.gen_range(10..(ARENA_WIDTH_TILES - 10)) as i32;
                let y = rng.gen_range(10..(ARENA_HEIGHT_TILES - 10)) as i32;
                let pos = TilePos::new(x, y);
                
                // Check if position is valid (simple check for initial spawn)
                let mut valid = true;
                
                // Check distance from mechs
                for mech in self.mechs.values() {
                    let dx = (pos.x - mech.position.x).abs();
                    let dy = (pos.y - mech.position.y).abs();
                    let distance = ((dx * dx + dy * dy) as f32).sqrt();
                    
                    if distance < 15.0 { // Keep initial resources away from mechs
                        valid = false;
                        break;
                    }
                }
                
                // Check distance from other resources
                if valid {
                    for resource in self.get_resources() {
                        let dx = (pos.x - resource.position.x).abs();
                        let dy = (pos.y - resource.position.y).abs();
                        let distance = ((dx * dx + dy * dy) as f32).sqrt();
                        
                        if distance < 5.0 { // Minimum spacing between resources
                            valid = false;
                            break;
                        }
                    }
                }
                
                if valid {
                    // Pick a random resource type
                    let resource_type = resource_types[rng.gen_range(0..resource_types.len())];
                    self.spawn_resource_with_behavior(pos, resource_type);
                    break;
                }
                
                attempts += 1;
            }
        }
    }

    pub fn add_player(&mut self, id: Uuid, name: String, preferred_team: Option<TeamId>) -> (TeamId, WorldPos) {
        // Balance teams
        let red_count = self.players.values().filter(|p| p.team == TeamId::Red).count();
        let blue_count = self.players.values().filter(|p| p.team == TeamId::Blue).count();
        
        let team = if let Some(pref) = preferred_team {
            if (red_count as i32 - blue_count as i32).abs() <= MAX_TEAM_SIZE_DIFFERENCE as i32 {
                pref
            } else if red_count < blue_count {
                TeamId::Red
            } else {
                TeamId::Blue
            }
        } else if red_count <= blue_count {
            TeamId::Red
        } else {
            TeamId::Blue
        };

        // Spawn near team mech (but not inside it!)
        let spawn_pos = match team {
            TeamId::Red => WorldPos::new(RED_PLAYER_SPAWN.0 * TILE_SIZE, RED_PLAYER_SPAWN.1 * TILE_SIZE),
            TeamId::Blue => WorldPos::new(BLUE_PLAYER_SPAWN.0 * TILE_SIZE, BLUE_PLAYER_SPAWN.1 * TILE_SIZE),
        };

        let player = Player {
            id,
            name,
            team,
            location: PlayerLocation::OutsideWorld(spawn_pos),
            carrying_resource: None,
            operating_station: None,
        };

        self.players.insert(id, player);
        (team, spawn_pos)
    }

    pub fn remove_player(&mut self, player_id: &Uuid) {
        // Exit any station they're operating
        if let Some(player) = self.players.get(player_id) {
            if let Some(station_id) = player.operating_station {
                for mech in self.mechs.values_mut() {
                    if let Some(station) = mech.stations.get_mut(&station_id) {
                        station.operated_by = None;
                    }
                }
            }
        }
        
        self.players.remove(player_id);
    }

    pub fn get_full_state(&self) -> ServerMessage {
        let players: HashMap<Uuid, PlayerState> = self.players.iter()
            .map(|(id, p)| (*id, PlayerState {
                id: p.id,
                name: p.name.clone(),
                team: p.team,
                location: p.location,
                carrying_resource: p.carrying_resource,
                operating_station: p.operating_station,
            }))
            .collect();

        let mechs: HashMap<Uuid, MechState> = self.mechs.iter()
            .map(|(id, m)| {
                let stations: Vec<StationState> = m.stations.values()
                    .map(|s| StationState {
                        id: s.id,
                        station_type: s.station_type,
                        floor: s.floor,
                        position: s.position,
                        operated_by: s.operated_by,
                    })
                    .collect();

                (*id, MechState {
                    id: m.id,
                    team: m.team,
                    position: m.position,
                    world_position: m.world_position,
                    health: m.health,
                    shield: m.shield,
                    upgrades: m.upgrades,
                    stations,
                    resource_inventory: m.resource_inventory.clone(),
                })
            })
            .collect();

        let resources: Vec<ResourceState> = self.get_resources().iter()
            .map(|r| ResourceState {
                id: r.id,
                position: r.position,
                resource_type: r.resource_type,
            })
            .collect();

        let projectiles: Vec<ProjectileState> = self.projectiles.values()
            .map(|p| ProjectileState {
                id: p.id,
                position: p.position,
                velocity: p.velocity,
                damage: p.damage,
                owner_mech_id: p.owner_mech_id,
            })
            .collect();

        ServerMessage::GameState {
            players,
            mechs,
            resources,
            projectiles,
        }
    }

    pub fn update_physics(&mut self, delta: f32) -> Vec<ServerMessage> {
        // Use the new pooled object update method
        self.update_pooled_objects(delta)
    }

    pub fn check_resource_pickups(&mut self, tx: &broadcast::Sender<(Uuid, ServerMessage)>) {
        let mut pickups = Vec::new();

        for player in self.players.values() {
            if player.carrying_resource.is_some() {
                continue;
            }

            if let PlayerLocation::OutsideWorld(player_pos) = player.location {
                let player_tile = player_pos.to_tile_pos();
                for resource in self.get_resources() {
                    if resource.position.distance_to(player_tile) < RESOURCE_PICKUP_DISTANCE {
                        pickups.push((player.id, resource.id, resource.resource_type));
                        break;
                    }
                }
            }
        }

        for (player_id, resource_id, resource_type) in pickups {
            if let Some(player) = self.players.get_mut(&player_id) {
                player.carrying_resource = Some(resource_type);
                self.remove_resource(resource_id);

                let msg = ServerMessage::PlayerPickedUpResource {
                    player_id,
                    resource_type,
                    resource_id,
                };
                let _ = tx.send((Uuid::nil(), msg));
                log::info!("Player {} picked up {:?} resource", player_id, resource_type);
            }
        }
    }

    pub fn check_mech_entries(&mut self, _tx: &broadcast::Sender<(Uuid, ServerMessage)>) {
        // Check if players can enter mechs
        // This is simplified - in full game would check for entrance points
    }

    pub fn update_projectiles(&mut self, _delta: f32, tx: &broadcast::Sender<(Uuid, ServerMessage)>) {
        // Check projectile collisions with mechs
        let mut hits = Vec::new();

        for projectile in self.projectiles.values() {
            let proj_tile = projectile.position.to_tile_pos();
            
            for mech in self.mechs.values() {
                if mech.id == projectile.owner_mech_id {
                    continue;
                }

                let mech_min = mech.position;
                let mech_max = mech.position.offset(MECH_SIZE_TILES, MECH_SIZE_TILES);

                if proj_tile.x >= mech_min.x && proj_tile.x <= mech_max.x &&
                   proj_tile.y >= mech_min.y && proj_tile.y <= mech_max.y {
                    hits.push((projectile.id, mech.id, projectile.damage));
                    break;
                }
            }
        }

        for (proj_id, mech_id, damage) in hits {
            self.projectiles.remove(&proj_id);
            
            if let Some(mech) = self.mechs.get_mut(&mech_id) {
                // Apply damage to shield first, then health
                let shield_damage = damage.min(mech.shield);
                mech.shield -= shield_damage;
                let health_damage = damage - shield_damage;
                mech.health = mech.health.saturating_sub(health_damage);

                let _ = tx.send((Uuid::nil(), ServerMessage::MechDamaged {
                    mech_id,
                    damage,
                    health_remaining: mech.health,
                }));

                let _ = tx.send((Uuid::nil(), ServerMessage::ProjectileHit {
                    projectile_id: proj_id,
                    hit_mech_id: Some(mech_id),
                    damage_dealt: damage,
                }));
            }
        }
    }
    
    pub fn update(&mut self, delta_time: f32) -> Vec<ServerMessage> {
        // Update tick count
        self.tick_count += 1;
        
        // Temporarily take the system manager to avoid borrowing issues
        let mut system_manager = std::mem::take(&mut self.system_manager);
        let messages = system_manager.update_all(self, delta_time);
        self.system_manager = system_manager;
        
        messages
    }
    
    /// Legacy update method - now handled by systems
    pub fn update_legacy(&mut self, delta_time: f32) -> Vec<ServerMessage> {
        let mut messages = Vec::new();
        
        // Update mech positions
        let mut mech_updates = Vec::new();
        for mech in self.mechs.values_mut() {
            if mech.velocity.0 != 0.0 || mech.velocity.1 != 0.0 {
                // Update world position
                mech.world_position.x += mech.velocity.0 * TILE_SIZE * delta_time;
                mech.world_position.y += mech.velocity.1 * TILE_SIZE * delta_time;
                
                // Keep in bounds
                mech.world_position.x = mech.world_position.x.max(0.0).min((ARENA_WIDTH_TILES as f32 - MECH_SIZE_TILES as f32) * TILE_SIZE);
                mech.world_position.y = mech.world_position.y.max(0.0).min((ARENA_HEIGHT_TILES as f32 - MECH_SIZE_TILES as f32) * TILE_SIZE);
                
                // Update tile position
                let new_tile_pos = mech.world_position.to_tile_pos();
                if new_tile_pos != mech.position {
                    mech.position = new_tile_pos;
                }
                
                mech_updates.push((mech.id, mech.position, mech.world_position));
            }
        }
        
        // Send mech position updates
        for (mech_id, position, world_position) in mech_updates {
            messages.push(ServerMessage::MechMoved { mech_id, position, world_position });
        }
        
        // Check for mech-player collisions (instant death)
        let mut killed_players = Vec::new();
        for (player_id, player) in self.players.iter() {
            if let PlayerLocation::OutsideWorld(player_pos) = player.location {
                for mech in self.mechs.values() {
                    // Check if player is within mech bounds
                    let mech_min_x = mech.world_position.x;
                    let mech_max_x = mech.world_position.x + (MECH_SIZE_TILES as f32 * TILE_SIZE);
                    let mech_min_y = mech.world_position.y;
                    let mech_max_y = mech.world_position.y + (MECH_SIZE_TILES as f32 * TILE_SIZE);
                    
                    if player_pos.x >= mech_min_x && player_pos.x <= mech_max_x &&
                       player_pos.y >= mech_min_y && player_pos.y <= mech_max_y {
                        // Player was run over!
                        killed_players.push(*player_id);
                        break;
                    }
                }
            }
        }
        
        // Handle killed players
        for player_id in killed_players {
            if let Some(player) = self.players.get(&player_id) {
                // Respawn at team spawn
                let spawn_pos = match player.team {
                    TeamId::Red => WorldPos::new(RED_PLAYER_SPAWN.0 * TILE_SIZE, RED_PLAYER_SPAWN.1 * TILE_SIZE),
                    TeamId::Blue => WorldPos::new(BLUE_PLAYER_SPAWN.0 * TILE_SIZE, BLUE_PLAYER_SPAWN.1 * TILE_SIZE),
                };
                
                messages.push(ServerMessage::PlayerKilled {
                    player_id,
                    killer: None, // Killed by mech
                    respawn_position: spawn_pos,
                });
                
                // Reset player state
                if let Some(player) = self.players.get_mut(&player_id) {
                    player.location = PlayerLocation::OutsideWorld(spawn_pos);
                    player.carrying_resource = None;
                    player.operating_station = None;
                }
            }
        }
        
        messages
    }
    
    /// Create a new projectile using the object pool
    pub fn create_projectile(
        &mut self,
        position: WorldPos,
        velocity: (f32, f32),
        damage: u32,
        owner_mech_id: Uuid,
        max_lifetime: f32,
    ) -> Uuid {
        let mut projectile = self.pool_manager.get_projectile();
        projectile.initialize(position, velocity, damage, owner_mech_id, max_lifetime);
        let projectile_id = projectile.id;
        self.projectiles.insert(projectile_id, projectile);
        projectile_id
    }
    
    /// Create a new visual effect using the object pool
    pub fn create_effect(
        &mut self,
        effect_type: EffectType,
        position: WorldPos,
        max_duration: f32,
        intensity: f32,
        color: (f32, f32, f32, f32),
    ) -> Uuid {
        let mut effect = self.pool_manager.get_effect();
        effect.initialize(effect_type, position, max_duration, intensity, color);
        let effect_id = effect.id;
        self.active_effects.insert(effect_id, effect);
        effect_id
    }
    
    /// Update pooled objects (projectiles and effects)
    pub fn update_pooled_objects(&mut self, delta_time: f32) -> Vec<ServerMessage> {
        let mut messages = Vec::new();
        
        // Update projectiles
        let mut projectiles_to_remove = Vec::new();
        for (id, projectile) in self.projectiles.iter_mut() {
            if !projectile.update(delta_time) {
                projectiles_to_remove.push(*id);
            }
        }
        
        // Remove expired projectiles and return them to pool
        for id in projectiles_to_remove {
            if let Some(mut projectile) = self.projectiles.remove(&id) {
                projectile.reset();
                self.pool_manager.return_projectile(projectile);
                
                messages.push(ServerMessage::ProjectileExpired {
                    projectile_id: id,
                });
            }
        }
        
        // Update effects
        let mut effects_to_remove = Vec::new();
        for (id, effect) in self.active_effects.iter_mut() {
            if !effect.update(delta_time) {
                effects_to_remove.push(*id);
            }
        }
        
        // Remove expired effects and return them to pool
        for id in effects_to_remove {
            if let Some(mut effect) = self.active_effects.remove(&id) {
                effect.reset();
                self.pool_manager.return_effect(effect);
                
                messages.push(ServerMessage::EffectExpired {
                    effect_id: id,
                });
            }
        }
        
        messages
    }
    
    /// Get pool statistics for monitoring
    pub fn get_pool_stats(&self) -> PoolStats {
        self.pool_manager.get_stats()
    }
    
    /// Clean up expired objects and optimize pools
    pub fn cleanup_pools(&mut self) {
        // This method can be called periodically to optimize memory usage
        // For now, the pools self-manage their size, but we could add
        // more sophisticated cleanup logic here if needed
        
        // Log pool statistics
        let stats = self.get_pool_stats();
        log::debug!(
            "Pool stats - Projectiles: {}/{}, Effects: {}/{}",
            stats.projectiles_available,
            stats.projectiles_max,
            stats.effects_available,
            stats.effects_max
        );
    }
}