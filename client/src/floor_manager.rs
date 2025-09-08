use shared::{
    mech_layout::{MechInterior, MechStation},
    tile_entity::{FloorMap, StaticTile},
    TilePos,
};
use std::collections::HashMap;
use uuid::Uuid;

/// Client-side floor data manager for mechs
/// Handles receiving and storing detailed floor data from the server
pub struct FloorManager {
    /// Cached floor data for each mech
    mech_floors: HashMap<Uuid, MechInterior>,
    /// Stations for each mech (keyed by station ID)
    mech_stations: HashMap<Uuid, HashMap<Uuid, MechStation>>,
}

impl FloorManager {
    pub fn new() -> Self {
        Self {
            mech_floors: HashMap::new(),
            mech_stations: HashMap::new(),
        }
    }

    /// Update floor data for a specific mech
    pub fn update_mech_floors(&mut self, mech_id: Uuid, floor_data: MechInterior, stations: HashMap<Uuid, MechStation>) {
        self.mech_floors.insert(mech_id, floor_data);
        self.mech_stations.insert(mech_id, stations);
    }

    /// Get floor data for a specific mech and floor
    pub fn get_floor(&self, mech_id: Uuid, floor: u8) -> Option<&FloorMap> {
        self.mech_floors
            .get(&mech_id)?
            .get_floor(floor)
    }

    /// Get all floor data for a mech
    pub fn get_mech_interior(&self, mech_id: Uuid) -> Option<&MechInterior> {
        self.mech_floors.get(&mech_id)
    }

    /// Get stations for a specific mech
    pub fn get_mech_stations(&self, mech_id: Uuid) -> Option<&HashMap<Uuid, MechStation>> {
        self.mech_stations.get(&mech_id)
    }

    /// Get stations on a specific floor of a mech
    pub fn get_floor_stations(&self, mech_id: Uuid, floor: u8) -> Vec<&MechStation> {
        if let Some(stations) = self.mech_stations.get(&mech_id) {
            stations.values()
                .filter(|station| station.floor == floor)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Check if a position is a valid stairway for floor transitions
    pub fn is_stairway_position(&self, mech_id: Uuid, floor: u8, position: TilePos) -> Option<u8> {
        if let Some(floor_map) = self.get_floor(mech_id, floor) {
            if let Some(StaticTile::TransitionZone { zone_id, .. }) = floor_map.static_tiles.get(&position) {
                return Some(*zone_id);
            }
        }
        None
    }

    /// Get all stairway positions on a floor
    pub fn get_stairway_positions(&self, mech_id: Uuid, floor: u8) -> Vec<(TilePos, u8)> {
        let mut stairways = Vec::new();
        if let Some(floor_map) = self.get_floor(mech_id, floor) {
            for (pos, tile) in &floor_map.static_tiles {
                if let StaticTile::TransitionZone { zone_id, .. } = tile {
                    stairways.push((*pos, *zone_id));
                }
            }
        }
        stairways
    }

    /// Check if player is on a station tile
    pub fn get_station_at_position(&self, mech_id: Uuid, floor: u8, position: TilePos) -> Option<&MechStation> {
        if let Some(floor_map) = self.get_floor(mech_id, floor) {
            // Check if there's a multi-tile station at this position
            if let Some(station_id) = floor_map.multi_tile_stations.get(&position) {
                return self.mech_stations.get(&mech_id)?.get(station_id);
            }
        }
        None
    }

    /// Get all station positions for a mech floor (for rendering)
    pub fn get_station_positions(&self, mech_id: Uuid, floor: u8) -> HashMap<TilePos, Uuid> {
        if let Some(floor_map) = self.get_floor(mech_id, floor) {
            floor_map.multi_tile_stations.clone()
        } else {
            HashMap::new()
        }
    }

    /// Check if a mech floor is loaded
    pub fn is_mech_loaded(&self, mech_id: Uuid) -> bool {
        self.mech_floors.contains_key(&mech_id)
    }

    /// Clear floor data for a mech (when mech is destroyed or out of range)
    pub fn clear_mech_data(&mut self, mech_id: Uuid) {
        self.mech_floors.remove(&mech_id);
        self.mech_stations.remove(&mech_id);
    }

    /// Get all loaded mech IDs
    pub fn get_loaded_mechs(&self) -> Vec<Uuid> {
        self.mech_floors.keys().cloned().collect()
    }

    /// Validate that floor data is consistent
    pub fn validate_mech_floors(&self, mech_id: Uuid) -> bool {
        if let Some(interior) = self.mech_floors.get(&mech_id) {
            // Check that all floors are present
            if interior.floors.len() != 3 {
                return false;
            }

            // Check that stations reference valid floors
            if let Some(stations) = self.mech_stations.get(&mech_id) {
                for station in stations.values() {
                    if station.floor >= 3 {
                        return false;
                    }
                    
                    // Check that station position is within floor bounds
                    if let Some(floor) = interior.get_floor(station.floor) {
                        let mut all_tiles_valid = true;
                        for dy in 0..station.size.height {
                            for dx in 0..station.size.width {
                                let tile_pos = TilePos::new(
                                    station.position.x + dx as i32,
                                    station.position.y + dy as i32
                                );
                                if !floor.multi_tile_stations.contains_key(&tile_pos) {
                                    all_tiles_valid = false;
                                    break;
                                }
                            }
                            if !all_tiles_valid {
                                break;
                            }
                        }
                        if !all_tiles_valid {
                            return false;
                        }
                    }
                }
            }

            return true;
        }
        false
    }
}

impl Default for FloorManager {
    fn default() -> Self {
        Self::new()
    }
}