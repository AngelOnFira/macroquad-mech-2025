use naia_server::{Server, EntityRef};
use shared::{components::*, messages::*, types::*, constants::*};
use crate::player_manager::PlayerManager;
use std::collections::HashMap;

pub struct GameWorld {
    mechs: HashMap<u16, MechData>,
    resources: Vec<ResourceData>,
    next_mech_id: u16,
    team_counts: HashMap<TeamId, usize>,
}

struct MechData {
    entity: EntityRef,
    position: TilePos,
    team: TeamId,
    interior_layout: MechInterior,
}

struct MechInterior {
    floors: Vec<FloorLayout>,
}

struct FloorLayout {
    tiles: Vec<Vec<bool>>, // true = walkable
    stations: Vec<(TilePos, StationType)>,
    ladders: Vec<TilePos>,
}

struct ResourceData {
    entity: EntityRef,
    position: TilePos,
    resource_type: ResourceType,
}

impl GameWorld {
    pub fn new() -> Self {
        let mut world = Self {
            mechs: HashMap::new(),
            resources: Vec::new(),
            next_mech_id: 0,
            team_counts: HashMap::new(),
        };
        
        world.team_counts.insert(TeamId::Red, 0);
        world.team_counts.insert(TeamId::Blue, 0);
        
        world
    }

    pub fn initialize(&mut self, server: &mut Server) {
        // Create initial mechs
        self.create_mech(server, TilePos::new(30, 30), TeamId::Red);
        self.create_mech(server, TilePos::new(70, 70), TeamId::Blue);
        
        // Create initial resources
        self.create_resource(server, TilePos::new(40, 40), ResourceType::ScrapMetal);
        self.create_resource(server, TilePos::new(60, 60), ResourceType::ComputerComponents);
        self.create_resource(server, TilePos::new(45, 55), ResourceType::Wiring);
        self.create_resource(server, TilePos::new(55, 45), ResourceType::Batteries);
    }

    fn create_mech(&mut self, server: &mut Server, position: TilePos, team: TeamId) -> u16 {
        let mech_id = self.next_mech_id;
        self.next_mech_id += 1;
        
        // Create mech entity
        let mech_entity = server.entity_spawn();
        let mech = Mech::new(position, team);
        server.entity_add_component(&mech_entity, mech);
        
        // Create mech interior layout
        let interior = self.create_mech_interior();
        
        // Create station entities
        for (floor_idx, floor) in interior.floors.iter().enumerate() {
            for (station_pos, station_type) in &floor.stations {
                let station_entity = server.entity_spawn();
                let station = Station::new_complete(
                    mech_id,
                    floor_idx as u8,
                    *station_pos,
                    *station_type,
                    None,
                );
                server.entity_add_component(&station_entity, station);
            }
        }
        
        // Create upgrade tracking entity
        let upgrade_entity = server.entity_spawn();
        let upgrades = MechUpgrade::new_complete(mech_id, 1, 1, 1, 1);
        server.entity_add_component(&upgrade_entity, upgrades);
        
        let mech_data = MechData {
            entity: mech_entity,
            position,
            team,
            interior_layout: interior,
        };
        
        self.mechs.insert(mech_id, mech_data);
        mech_id
    }

    fn create_mech_interior(&self) -> MechInterior {
        let mut floors = Vec::new();
        
        for floor_level in 0..MECH_FLOORS {
            let mut tiles = vec![vec![false; FLOOR_WIDTH_TILES as usize]; FLOOR_HEIGHT_TILES as usize];
            let mut stations = Vec::new();
            let mut ladders = Vec::new();
            
            // Make interior walkable (except walls)
            for y in 1..(FLOOR_HEIGHT_TILES - 1) {
                for x in 1..(FLOOR_WIDTH_TILES - 1) {
                    tiles[y as usize][x as usize] = true;
                }
            }
            
            // Add ladders
            if floor_level < MECH_FLOORS - 1 {
                ladders.push(TilePos::new(1, 1));
                ladders.push(TilePos::new(FLOOR_WIDTH_TILES - 2, FLOOR_HEIGHT_TILES - 2));
            }
            
            // Add stations based on floor
            match floor_level {
                0 => {
                    stations.push((TilePos::new(5, 3), StationType::Engine));
                    stations.push((TilePos::new(10, 3), StationType::Electrical));
                }
                1 => {
                    stations.push((TilePos::new(5, 3), StationType::WeaponLaser));
                    stations.push((TilePos::new(10, 3), StationType::WeaponProjectile));
                    stations.push((TilePos::new(15, 3), StationType::Shield));
                }
                2 => {
                    stations.push((TilePos::new(5, 3), StationType::Repair));
                    stations.push((TilePos::new(10, 3), StationType::Upgrade));
                }
                _ => {}
            }
            
            floors.push(FloorLayout {
                tiles,
                stations,
                ladders,
            });
        }
        
        MechInterior { floors }
    }

    fn create_resource(&mut self, server: &mut Server, position: TilePos, resource_type: ResourceType) {
        let resource_entity = server.entity_spawn();
        let resource = Resource::new_complete(position, resource_type);
        server.entity_add_component(&resource_entity, resource);
        
        self.resources.push(ResourceData {
            entity: resource_entity,
            position,
            resource_type,
        });
    }

    pub fn assign_team(&mut self, preferred: Option<TeamId>) -> TeamId {
        let red_count = *self.team_counts.get(&TeamId::Red).unwrap_or(&0);
        let blue_count = *self.team_counts.get(&TeamId::Blue).unwrap_or(&0);
        
        let team = if let Some(pref) = preferred {
            // Use preference if teams are balanced
            if (red_count as i32 - blue_count as i32).abs() <= 1 {
                pref
            } else if red_count < blue_count {
                TeamId::Red
            } else {
                TeamId::Blue
            }
        } else {
            // Auto-assign to smaller team
            if red_count <= blue_count {
                TeamId::Red
            } else {
                TeamId::Blue
            }
        };
        
        *self.team_counts.entry(team).or_insert(0) += 1;
        team
    }

    pub fn get_spawn_position(&self, team: TeamId) -> TilePos {
        // Spawn near team's mech
        match team {
            TeamId::Red => TilePos::new(25, 25),
            TeamId::Blue => TilePos::new(75, 75),
        }
    }

    pub fn update(&mut self, server: &mut Server, player_manager: &mut PlayerManager) {
        // Game update logic here
        // For now, just periodic resource respawning could go here
    }

    pub fn handle_player_movement(
        &mut self,
        server: &mut Server,
        player_entity_id: u16,
        input: &PlayerInput,
    ) {
        // Get player entity and component
        // Note: In real implementation, we'd need to track entities properly
        // For now, this is a simplified version
        
        if let Some(direction) = input.direction {
            // Calculate new position based on direction
            // Update player location component
            // Check collision with mech entrances, resources, etc.
            
            // This would involve:
            // 1. Getting current player location
            // 2. Calculating new position
            // 3. Validating movement (collision, distance from mech if tethered)
            // 4. Updating player component
            // 5. Checking for interactions (resource pickup, mech entry)
        }
        
        if input.action_key_pressed {
            // Handle action based on player's current location
            // - Pick up resource if near one
            // - Enter mech if at entrance
            // - Enter station if inside mech and near station
            // - Exit mech if inside
        }
    }

    pub fn handle_station_interaction(
        &mut self,
        server: &mut Server,
        player_entity_id: u16,
        input: &StationInput,
    ) {
        // Handle button presses at stations
        if let Some(button) = input.button_pressed {
            // Process station-specific actions
            // Fire weapons, adjust shields, repair, etc.
        }
    }
}