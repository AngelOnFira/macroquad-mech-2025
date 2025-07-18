use std::collections::HashMap;
use uuid::Uuid;
use shared::{components::*, tile_entity::*, types::{TilePos, WorldPos}};

// =============================================================================
// Entity Storage System
// =============================================================================

pub struct EntityStorage {
    // Component storage - each component type has its own HashMap
    positions: HashMap<Uuid, Position>,
    stations: HashMap<Uuid, Station>,
    turrets: HashMap<Uuid, Turret>,
    power_nodes: HashMap<Uuid, PowerNode>,
    power_consumers: HashMap<Uuid, PowerConsumer>,
    power_producers: HashMap<Uuid, PowerProducer>,
    breakables: HashMap<Uuid, Breakable>,
    renderables: HashMap<Uuid, Renderable>,
    interactables: HashMap<Uuid, Interactable>,
    solids: HashMap<Uuid, Solid>,
    opaques: HashMap<Uuid, Opaque>,
    oxygen_producers: HashMap<Uuid, OxygenProducer>,
    resource_storages: HashMap<Uuid, ResourceStorage>,
    scriptables: HashMap<Uuid, Scriptable>,
    
    // Entity tracking
    entities: HashMap<Uuid, EntityInfo>,
    
    // Spatial indices for fast queries
    entities_by_position: HashMap<TilePos, Vec<Uuid>>,
    entities_by_mech: HashMap<Uuid, Vec<Uuid>>,
}

#[derive(Debug, Clone)]
struct EntityInfo {
    id: Uuid,
    name: String,
    active: bool,
}

impl EntityStorage {
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
            stations: HashMap::new(),
            turrets: HashMap::new(),
            power_nodes: HashMap::new(),
            power_consumers: HashMap::new(),
            power_producers: HashMap::new(),
            breakables: HashMap::new(),
            renderables: HashMap::new(),
            interactables: HashMap::new(),
            solids: HashMap::new(),
            opaques: HashMap::new(),
            oxygen_producers: HashMap::new(),
            resource_storages: HashMap::new(),
            scriptables: HashMap::new(),
            entities: HashMap::new(),
            entities_by_position: HashMap::new(),
            entities_by_mech: HashMap::new(),
        }
    }
    
    // =============================================================================
    // Entity Management
    // =============================================================================
    
    pub fn create_entity(&mut self, name: String) -> Uuid {
        let id = Uuid::new_v4();
        self.entities.insert(id, EntityInfo {
            id,
            name,
            active: true,
        });
        id
    }
    
    pub fn spawn_from_template(&mut self, template: &EntityTemplate, position: Position) -> Uuid {
        let entity_id = self.create_entity(template.name.clone());
        
        // Add position component and update spatial index
        self.add_position(entity_id, position);
        
        // Add all components from template
        if let Some(station) = &template.components.station {
            self.stations.insert(entity_id, station.clone());
        }
        if let Some(turret) = &template.components.turret {
            self.turrets.insert(entity_id, turret.clone());
        }
        if let Some(power_node) = &template.components.power_node {
            self.power_nodes.insert(entity_id, power_node.clone());
        }
        if let Some(power_consumer) = &template.components.power_consumer {
            self.power_consumers.insert(entity_id, power_consumer.clone());
        }
        if let Some(power_producer) = &template.components.power_producer {
            self.power_producers.insert(entity_id, power_producer.clone());
        }
        if let Some(breakable) = &template.components.breakable {
            self.breakables.insert(entity_id, breakable.clone());
        }
        if let Some(renderable) = &template.components.renderable {
            self.renderables.insert(entity_id, renderable.clone());
        }
        if let Some(interactable) = &template.components.interactable {
            self.interactables.insert(entity_id, interactable.clone());
        }
        if let Some(solid) = &template.components.solid {
            self.solids.insert(entity_id, solid.clone());
        }
        if let Some(opaque) = &template.components.opaque {
            self.opaques.insert(entity_id, opaque.clone());
        }
        if let Some(oxygen_producer) = &template.components.oxygen_producer {
            self.oxygen_producers.insert(entity_id, oxygen_producer.clone());
        }
        if let Some(resource_storage) = &template.components.resource_storage {
            self.resource_storages.insert(entity_id, resource_storage.clone());
        }
        if let Some(scriptable) = &template.components.scriptable {
            self.scriptables.insert(entity_id, scriptable.clone());
        }
        
        entity_id
    }
    
    pub fn destroy_entity(&mut self, entity_id: Uuid) {
        // Remove from all component storages
        if let Some(pos) = self.positions.remove(&entity_id) {
            // Update spatial index
            if let Some(entities) = self.entities_by_position.get_mut(&pos.tile) {
                entities.retain(|&id| id != entity_id);
            }
            if let Some(mech_id) = pos.mech_id {
                if let Some(entities) = self.entities_by_mech.get_mut(&mech_id) {
                    entities.retain(|&id| id != entity_id);
                }
            }
        }
        
        self.stations.remove(&entity_id);
        self.turrets.remove(&entity_id);
        self.power_nodes.remove(&entity_id);
        self.power_consumers.remove(&entity_id);
        self.power_producers.remove(&entity_id);
        self.breakables.remove(&entity_id);
        self.renderables.remove(&entity_id);
        self.interactables.remove(&entity_id);
        self.solids.remove(&entity_id);
        self.opaques.remove(&entity_id);
        self.oxygen_producers.remove(&entity_id);
        self.resource_storages.remove(&entity_id);
        self.scriptables.remove(&entity_id);
        
        self.entities.remove(&entity_id);
    }
    
    // =============================================================================
    // Position Management with Spatial Indexing
    // =============================================================================
    
    fn add_position(&mut self, entity_id: Uuid, position: Position) {
        // Update spatial indices
        self.entities_by_position
            .entry(position.tile)
            .or_default()
            .push(entity_id);
            
        if let Some(mech_id) = position.mech_id {
            self.entities_by_mech
                .entry(mech_id)
                .or_default()
                .push(entity_id);
        }
        
        self.positions.insert(entity_id, position);
    }
    
    pub fn update_position(&mut self, entity_id: Uuid, new_position: Position) {
        // Remove from old indices
        if let Some(old_pos) = self.positions.get(&entity_id) {
            if let Some(entities) = self.entities_by_position.get_mut(&old_pos.tile) {
                entities.retain(|&id| id != entity_id);
            }
            if let Some(mech_id) = old_pos.mech_id {
                if let Some(entities) = self.entities_by_mech.get_mut(&mech_id) {
                    entities.retain(|&id| id != entity_id);
                }
            }
        }
        
        // Add to new indices
        self.add_position(entity_id, new_position);
    }
    
    // =============================================================================
    // Spatial Queries
    // =============================================================================
    
    pub fn get_entities_at_tile(&self, tile: TilePos) -> Vec<Uuid> {
        self.entities_by_position
            .get(&tile)
            .cloned()
            .unwrap_or_default()
    }
    
    pub fn get_entities_in_mech(&self, mech_id: Uuid) -> Vec<Uuid> {
        self.entities_by_mech
            .get(&mech_id)
            .cloned()
            .unwrap_or_default()
    }
    
    pub fn get_entities_in_radius(&self, center: WorldPos, radius: f32) -> Vec<Uuid> {
        let mut result = Vec::new();
        let radius_squared = radius * radius;
        
        for (&entity_id, pos) in &self.positions {
            let dx = pos.world.x - center.x;
            let dy = pos.world.y - center.y;
            if dx * dx + dy * dy <= radius_squared {
                result.push(entity_id);
            }
        }
        
        result
    }
    
    // =============================================================================
    // Component Queries
    // =============================================================================
    
    pub fn get_stations_in_mech(&self, mech_id: Uuid) -> Vec<(Uuid, &Station, &Position)> {
        let mut result = Vec::new();
        
        for entity_id in self.get_entities_in_mech(mech_id) {
            if let (Some(station), Some(pos)) = (
                self.stations.get(&entity_id),
                self.positions.get(&entity_id)
            ) {
                result.push((entity_id, station, pos));
            }
        }
        
        result
    }
    
    pub fn get_turrets_in_range(&self, center: WorldPos, range: f32) -> Vec<(Uuid, &Turret, &Position)> {
        let mut result = Vec::new();
        
        for entity_id in self.get_entities_in_radius(center, range) {
            if let (Some(turret), Some(pos)) = (
                self.turrets.get(&entity_id),
                self.positions.get(&entity_id)
            ) {
                result.push((entity_id, turret, pos));
            }
        }
        
        result
    }
}

// =============================================================================
// Component Storage Trait Implementation
// =============================================================================

impl ComponentStorage for EntityStorage {
    fn get_position(&self, entity: Uuid) -> Option<&Position> {
        self.positions.get(&entity)
    }
    
    fn get_station(&self, entity: Uuid) -> Option<&Station> {
        self.stations.get(&entity)
    }
    
    fn get_renderable(&self, entity: Uuid) -> Option<&Renderable> {
        self.renderables.get(&entity)
    }
    
    fn get_solid(&self, entity: Uuid) -> Option<&Solid> {
        self.solids.get(&entity)
    }
    
    fn get_opaque(&self, entity: Uuid) -> Option<&Opaque> {
        self.opaques.get(&entity)
    }
    
    fn get_position_mut(&mut self, entity: Uuid) -> Option<&mut Position> {
        self.positions.get_mut(&entity)
    }
    
    fn get_station_mut(&mut self, entity: Uuid) -> Option<&mut Station> {
        self.stations.get_mut(&entity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::{StationType, TilePos, WorldPos};
    
    #[test]
    fn test_entity_creation_and_destruction() {
        let mut storage = EntityStorage::new();
        
        let entity_id = storage.create_entity("Test Entity".to_string());
        assert!(storage.entities.contains_key(&entity_id));
        
        storage.destroy_entity(entity_id);
        assert!(!storage.entities.contains_key(&entity_id));
    }
    
    #[test]
    fn test_spawn_from_template() {
        let mut storage = EntityStorage::new();
        
        let template = EntityTemplate {
            name: "Test Station".to_string(),
            components: EntityComponents {
                station: Some(Station {
                    station_type: StationType::WeaponLaser,
                    interaction_range: 1.5,
                    power_required: 50.0,
                    operating: false,
                }),
                solid: Some(Solid {
                    blocks_movement: true,
                    blocks_projectiles: false,
                }),
                ..Default::default()
            },
        };
        
        let position = Position {
            tile: TilePos::new(5, 5),
            world: WorldPos::new(80.0, 80.0),
            floor: Some(1),
            mech_id: Some(Uuid::new_v4()),
        };
        
        let entity_id = storage.spawn_from_template(&template, position);
        
        assert!(storage.get_station(entity_id).is_some());
        assert!(storage.get_solid(entity_id).is_some());
        assert!(storage.get_position(entity_id).is_some());
    }
    
    #[test]
    fn test_spatial_queries() {
        let mut storage = EntityStorage::new();
        let mech_id = Uuid::new_v4();
        
        // Create entities at different positions
        let entity1 = storage.create_entity("Entity 1".to_string());
        storage.add_position(entity1, Position {
            tile: TilePos::new(5, 5),
            world: WorldPos::new(80.0, 80.0),
            floor: Some(1),
            mech_id: Some(mech_id),
        });
        
        let entity2 = storage.create_entity("Entity 2".to_string());
        storage.add_position(entity2, Position {
            tile: TilePos::new(5, 5),
            world: WorldPos::new(80.0, 80.0),
            floor: Some(1),
            mech_id: Some(mech_id),
        });
        
        let entity3 = storage.create_entity("Entity 3".to_string());
        storage.add_position(entity3, Position {
            tile: TilePos::new(10, 10),
            world: WorldPos::new(160.0, 160.0),
            floor: None,
            mech_id: None,
        });
        
        // Test queries
        let entities_at_tile = storage.get_entities_at_tile(TilePos::new(5, 5));
        assert_eq!(entities_at_tile.len(), 2);
        assert!(entities_at_tile.contains(&entity1));
        assert!(entities_at_tile.contains(&entity2));
        
        let entities_in_mech = storage.get_entities_in_mech(mech_id);
        assert_eq!(entities_in_mech.len(), 2);
        
        let entities_in_radius = storage.get_entities_in_radius(WorldPos::new(80.0, 80.0), 20.0);
        assert!(entities_in_radius.contains(&entity1));
        assert!(entities_in_radius.contains(&entity2));
        assert!(!entities_in_radius.contains(&entity3));
    }
}