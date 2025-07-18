use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use uuid::Uuid;
use crate::{TilePos, WorldPos, StationType, ResourceType};

// =============================================================================
// Position and Spatial Components
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub tile: TilePos,
    pub world: WorldPos,
    pub floor: Option<u8>, // None = outside, Some(n) = mech floor
    pub mech_id: Option<Uuid>,
}

// =============================================================================
// Station Components
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Station {
    pub station_type: StationType,
    pub interaction_range: f32,
    pub power_required: f32,
    pub operating: bool,
}

// =============================================================================
// Combat Components
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turret {
    pub damage: f32,
    pub fire_rate: f32,
    pub range: f32,
    pub ammo: u32,
    pub target_mode: TargetMode,
    pub current_target: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TargetMode {
    Nearest,
    LowestHealth,
    HighestThreat,
    Manual,
}

// =============================================================================
// Infrastructure Components
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerNode {
    pub max_throughput: f32,
    pub current_load: f32,
    pub connections: Vec<Uuid>,
    pub network_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerConsumer {
    pub idle_draw: f32,
    pub active_draw: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerProducer {
    pub output: f32,
    pub efficiency: f32,
}

// =============================================================================
// Damage and Health Components
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakable {
    pub health: f32,
    pub max_health: f32,
    pub armor: f32,
    pub break_effects: Vec<BreakEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BreakEffect {
    Explosion { radius: f32, damage: f32 },
    DebrisSpawn { count: u32 },
    PowerShutdown,
    GasLeak { gas_type: String, rate: f32 },
}

// =============================================================================
// Visual Components
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Renderable {
    pub sprite: SpriteId,
    pub layer: RenderLayer,
    pub color_modulation: Color,
    pub animation_state: Option<AnimationState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpriteId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderLayer {
    Floor,
    FloorDecal,
    Object,
    Wall,
    Overhead,
    Effect,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationState {
    pub animation_id: String,
    pub frame: u32,
    pub elapsed: f32,
    pub looping: bool,
}

// =============================================================================
// Interaction Components
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interactable {
    pub prompt: String,
    pub range: f32,
    pub requires_facing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Solid {
    pub blocks_movement: bool,
    pub blocks_projectiles: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opaque {
    pub blocks_completely: bool,
    pub attenuation: f32, // 0.0 = transparent, 1.0 = opaque
}

// =============================================================================
// Specialized Components
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OxygenProducer {
    pub rate: f32, // Oxygen units per second
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStorage {
    pub capacity: HashMap<ResourceType, u32>,
    pub current: HashMap<ResourceType, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scriptable {
    pub script_id: String,
    pub state: HashMap<String, serde_json::Value>,
}

// =============================================================================
// Entity Definition for Spawning
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityTemplate {
    pub name: String,
    pub components: EntityComponents,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntityComponents {
    pub position: Option<Position>,
    pub station: Option<Station>,
    pub turret: Option<Turret>,
    pub power_node: Option<PowerNode>,
    pub power_consumer: Option<PowerConsumer>,
    pub power_producer: Option<PowerProducer>,
    pub breakable: Option<Breakable>,
    pub renderable: Option<Renderable>,
    pub interactable: Option<Interactable>,
    pub solid: Option<Solid>,
    pub opaque: Option<Opaque>,
    pub oxygen_producer: Option<OxygenProducer>,
    pub resource_storage: Option<ResourceStorage>,
    pub scriptable: Option<Scriptable>,
}

// =============================================================================
// Component Queries (for ECS-like usage)
// =============================================================================

pub trait ComponentStorage {
    fn get_position(&self, entity: Uuid) -> Option<&Position>;
    fn get_station(&self, entity: Uuid) -> Option<&Station>;
    fn get_renderable(&self, entity: Uuid) -> Option<&Renderable>;
    fn get_solid(&self, entity: Uuid) -> Option<&Solid>;
    fn get_opaque(&self, entity: Uuid) -> Option<&Opaque>;
    
    fn get_position_mut(&mut self, entity: Uuid) -> Option<&mut Position>;
    fn get_station_mut(&mut self, entity: Uuid) -> Option<&mut Station>;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_entity_template_creation() {
        let template = EntityTemplate {
            name: "Laser Turret".to_string(),
            components: EntityComponents {
                turret: Some(Turret {
                    damage: 25.0,
                    fire_rate: 0.5,
                    range: 50.0,
                    ammo: 1000,
                    target_mode: TargetMode::Nearest,
                    current_target: None,
                }),
                power_consumer: Some(PowerConsumer {
                    idle_draw: 10.0,
                    active_draw: 50.0,
                }),
                solid: Some(Solid {
                    blocks_movement: true,
                    blocks_projectiles: false,
                }),
                ..Default::default()
            },
        };
        
        assert_eq!(template.name, "Laser Turret");
        assert!(template.components.turret.is_some());
        assert!(template.components.power_consumer.is_some());
    }
}