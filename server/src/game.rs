use std::collections::HashMap;
use tokio::sync::broadcast;
use uuid::Uuid;

use shared::*;

pub struct Game {
    pub players: HashMap<Uuid, Player>,
    pub mechs: HashMap<Uuid, Mech>,
    pub resources: HashMap<Uuid, Resource>,
    pub projectiles: HashMap<Uuid, Projectile>,
    pub tick_count: u64,
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
    pub stations: HashMap<Uuid, Station>,
    pub interior: MechInterior,
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

pub struct Station {
    pub id: Uuid,
    pub station_type: StationType,
    pub floor: u8,
    pub position: TilePos,
    pub operated_by: Option<Uuid>,
}

#[derive(Clone)]
pub struct MechInterior {
    pub floors: Vec<FloorLayout>,
}

#[derive(Clone)]
pub struct FloorLayout {
    pub tiles: Vec<Vec<TileType>>,
    pub ladders: Vec<TilePos>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum TileType {
    Empty,
    Floor,
    Wall,
    Station(StationType),
    Ladder,
}

pub struct Resource {
    pub id: Uuid,
    pub position: TilePos,
    pub resource_type: ResourceType,
}

pub struct Projectile {
    pub id: Uuid,
    pub position: WorldPos,
    pub velocity: (f32, f32),
    pub damage: u32,
    pub owner_mech_id: Uuid,
    pub lifetime: f32,
}

impl Game {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            mechs: HashMap::new(),
            resources: HashMap::new(),
            projectiles: HashMap::new(),
            tick_count: 0,
        }
    }

    pub fn create_initial_mechs(&mut self) {
        // Red team mech
        let red_mech = self.create_mech(TilePos::new(20, 20), TeamId::Red);
        self.mechs.insert(red_mech.id, red_mech);

        // Blue team mech
        let blue_mech = self.create_mech(TilePos::new(80, 80), TeamId::Blue);
        self.mechs.insert(blue_mech.id, blue_mech);
    }

    fn create_mech(&self, position: TilePos, team: TeamId) -> Mech {
        let id = Uuid::new_v4();
        let mut stations = HashMap::new();
        let interior = self.create_mech_interior(&mut stations, id);

        Mech {
            id,
            team,
            position,
            health: 100,
            max_health: 100,
            shield: 50,
            max_shield: 50,
            upgrades: MechUpgrades {
                laser_level: 1,
                projectile_level: 1,
                engine_level: 1,
                shield_level: 1,
            },
            stations,
            interior,
        }
    }

    fn create_mech_interior(&self, stations: &mut HashMap<Uuid, Station>, _mech_id: Uuid) -> MechInterior {
        let mut floors = Vec::new();

        for floor_idx in 0..MECH_FLOORS {
            let mut tiles = vec![vec![TileType::Empty; FLOOR_WIDTH_TILES as usize]; FLOOR_HEIGHT_TILES as usize];
            let mut ladders = Vec::new();

            // Create walls and floors
            for y in 0..FLOOR_HEIGHT_TILES {
                for x in 0..FLOOR_WIDTH_TILES {
                    if x == 0 || x == FLOOR_WIDTH_TILES - 1 || y == 0 || y == FLOOR_HEIGHT_TILES - 1 {
                        tiles[y as usize][x as usize] = TileType::Wall;
                    } else {
                        tiles[y as usize][x as usize] = TileType::Floor;
                    }
                }
            }

            // Add ladders between floors
            if floor_idx < MECH_FLOORS - 1 {
                let ladder1 = TilePos::new(2, 2);
                let ladder2 = TilePos::new(FLOOR_WIDTH_TILES - 3, FLOOR_HEIGHT_TILES - 3);
                tiles[ladder1.y as usize][ladder1.x as usize] = TileType::Ladder;
                tiles[ladder2.y as usize][ladder2.x as usize] = TileType::Ladder;
                ladders.push(ladder1);
                ladders.push(ladder2);
            }

            // Add stations based on floor
            let floor_stations = match floor_idx {
                0 => vec![
                    (TilePos::new(5, 3), StationType::Engine),
                    (TilePos::new(10, 3), StationType::Electrical),
                    (TilePos::new(15, 3), StationType::Upgrade),
                ],
                1 => vec![
                    (TilePos::new(5, 3), StationType::WeaponLaser),
                    (TilePos::new(10, 3), StationType::WeaponProjectile),
                    (TilePos::new(15, 3), StationType::Shield),
                ],
                2 => vec![
                    (TilePos::new(8, 3), StationType::Repair),
                ],
                _ => vec![],
            };

            for (pos, station_type) in floor_stations {
                tiles[pos.y as usize][pos.x as usize] = TileType::Station(station_type);
                let station = Station {
                    id: Uuid::new_v4(),
                    station_type,
                    floor: floor_idx as u8,
                    position: pos,
                    operated_by: None,
                };
                stations.insert(station.id, station);
            }

            floors.push(FloorLayout { tiles, ladders });
        }

        MechInterior { floors }
    }

    pub fn spawn_initial_resources(&mut self) {
        let resource_spawns = vec![
            (TilePos::new(40, 30), ResourceType::ScrapMetal),
            (TilePos::new(60, 30), ResourceType::ComputerComponents),
            (TilePos::new(30, 60), ResourceType::Wiring),
            (TilePos::new(70, 60), ResourceType::Batteries),
            (TilePos::new(50, 50), ResourceType::ScrapMetal),
        ];

        for (pos, resource_type) in resource_spawns {
            let resource = Resource {
                id: Uuid::new_v4(),
                position: pos,
                resource_type,
            };
            self.resources.insert(resource.id, resource);
        }
    }

    pub fn add_player(&mut self, id: Uuid, name: String, preferred_team: Option<TeamId>) -> (TeamId, WorldPos) {
        // Balance teams
        let red_count = self.players.values().filter(|p| p.team == TeamId::Red).count();
        let blue_count = self.players.values().filter(|p| p.team == TeamId::Blue).count();
        
        let team = if let Some(pref) = preferred_team {
            if (red_count as i32 - blue_count as i32).abs() <= 1 {
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
            TeamId::Red => WorldPos::new(15.0 * TILE_SIZE, 20.0 * TILE_SIZE), // Left of red mech at (20,20)
            TeamId::Blue => WorldPos::new(75.0 * TILE_SIZE, 80.0 * TILE_SIZE), // Left of blue mech at (80,80)
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
                    health: m.health,
                    shield: m.shield,
                    upgrades: m.upgrades,
                    stations,
                })
            })
            .collect();

        let resources: Vec<ResourceState> = self.resources.values()
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

    pub fn update_physics(&mut self, delta: f32) {
        // Update projectiles
        let projectiles_to_remove: Vec<Uuid> = self.projectiles.iter_mut()
            .filter_map(|(id, proj)| {
                proj.position.x += proj.velocity.0 * delta;
                proj.position.y += proj.velocity.1 * delta;
                proj.lifetime -= delta;
                
                if proj.lifetime <= 0.0 {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();

        for id in projectiles_to_remove {
            self.projectiles.remove(&id);
        }
    }

    pub fn check_resource_pickups(&mut self, tx: &broadcast::Sender<(Uuid, ServerMessage)>) {
        let mut pickups = Vec::new();

        for player in self.players.values() {
            if player.carrying_resource.is_some() {
                continue;
            }

            if let PlayerLocation::OutsideWorld(player_pos) = player.location {
                let player_tile = player_pos.to_tile_pos();
                for resource in self.resources.values() {
                    if resource.position.distance_to(&player_tile) < 1.5 {
                        pickups.push((player.id, resource.id, resource.resource_type));
                        break;
                    }
                }
            }
        }

        for (player_id, resource_id, resource_type) in pickups {
            if let Some(player) = self.players.get_mut(&player_id) {
                player.carrying_resource = Some(resource_type);
                self.resources.remove(&resource_id);

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
}