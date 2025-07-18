use std::collections::HashMap;
use uuid::Uuid;
use crate::{StationType, TilePos, ResourceType, GameResult, GameError, uuid_gen::new_uuid};

/// Registry pattern for managing station types and their behaviors
pub struct StationRegistry {
    station_definitions: HashMap<StationType, StationDefinition>,
}

/// Definition of a station type with its properties and behaviors
#[derive(Debug, Clone)]
pub struct StationDefinition {
    pub station_type: StationType,
    pub name: String,
    pub description: String,
    pub button_count: u8,
    pub button_definitions: Vec<ButtonDefinition>,
    pub cooldown_seconds: f32,
    pub resource_requirements: HashMap<ResourceType, u32>,
    pub upgrade_requirements: HashMap<ResourceType, u32>,
    pub allowed_floors: Vec<u8>,
    pub max_per_mech: u8,
    pub size: (u8, u8), // width, height in tiles
}

/// Definition of a button on a station
#[derive(Debug, Clone)]
pub struct ButtonDefinition {
    pub index: u8,
    pub label: String,
    pub description: String,
    pub action: StationAction,
    pub cooldown_seconds: f32,
    pub resource_cost: HashMap<ResourceType, u32>,
}

/// Actions that stations can perform
#[derive(Debug, Clone)]
pub enum StationAction {
    /// Fire a weapon at the nearest enemy
    FireWeapon {
        weapon_type: WeaponType,
        damage: u32,
        range: f32,
        speed: Option<f32>, // None for instant (laser), Some for projectile
    },
    /// Boost shield by a fixed amount
    BoostShield {
        amount: u32,
    },
    /// Repair mech health
    RepairMech {
        hp_per_resource: u32,
    },
    /// Upgrade a mech system
    UpgradeMech {
        upgrade_type: MechUpgradeType,
    },
    /// Charge energy reserves
    ChargeEnergy {
        energy_per_tick: u32,
    },
    /// Trigger a temporary effect
    TriggerEffect {
        effect: String, // We'll use String for now instead of EffectType
        duration: f32,
    },
    /// No action (placeholder)
    None,
}

/// Types of weapons
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponType {
    Laser,
    Projectile,
    Missile,
    Beam,
}

/// Types of mech upgrades
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MechUpgradeType {
    Laser,
    Projectile,
    Shield,
    Engine,
    Armor,
}

/// Station instance in a mech
#[derive(Debug, Clone)]
pub struct StationInstance {
    pub id: Uuid,
    pub station_type: StationType,
    pub floor: u8,
    pub position: TilePos,
    pub operated_by: Option<Uuid>,
    pub last_used: f32, // Time since last use for cooldown
    pub health: u32,
    pub max_health: u32,
    pub upgrade_level: u8,
}

/// Station button press result
#[derive(Debug, Clone)]
pub struct StationActionResult {
    pub success: bool,
    pub message: String,
    pub effects: Vec<StationEffect>,
    pub cooldown_applied: f32,
}

/// Effects that station actions can produce
#[derive(Debug, Clone)]
pub enum StationEffect {
    /// Damage dealt to a target
    Damage {
        target_id: Uuid,
        amount: u32,
    },
    /// Healing applied to a target
    Heal {
        target_id: Uuid,
        amount: u32,
    },
    /// Shield boost applied
    ShieldBoost {
        target_id: Uuid,
        amount: u32,
    },
    /// Resources consumed
    ResourceConsumed {
        resource_type: ResourceType,
        amount: u32,
    },
    /// Upgrade applied
    UpgradeApplied {
        target_id: Uuid,
        upgrade_type: MechUpgradeType,
        new_level: u8,
    },
    /// Projectile created
    ProjectileCreated {
        projectile_id: Uuid,
        position: crate::WorldPos,
        velocity: (f32, f32),
        damage: u32,
    },
    /// Visual effect triggered
    VisualEffect {
        effect_type: String,
        position: crate::WorldPos,
        duration: f32,
    },
    /// Energy charge
    EnergyCharge {
        target_id: Uuid,
        amount: u32,
    },
    /// Temporary buff applied
    TemporaryBuff {
        target_id: Uuid,
        buff_type: String,
        duration: f32,
    },
}

impl StationRegistry {
    /// Create a new station registry with default definitions
    pub fn new() -> Self {
        let mut registry = Self {
            station_definitions: HashMap::new(),
        };
        
        registry.register_default_stations();
        registry
    }
    
    /// Register a new station type
    pub fn register_station(&mut self, definition: StationDefinition) {
        self.station_definitions.insert(definition.station_type, definition);
    }
    
    /// Get a station definition by type
    pub fn get_definition(&self, station_type: StationType) -> Option<&StationDefinition> {
        self.station_definitions.get(&station_type)
    }
    
    /// Get all station definitions
    pub fn get_all_definitions(&self) -> Vec<&StationDefinition> {
        self.station_definitions.values().collect()
    }
    
    /// Create a new station instance
    pub fn create_station(&self, station_type: StationType, floor: u8, position: TilePos) -> GameResult<StationInstance> {
        let _definition = self.get_definition(station_type)
            .ok_or_else(|| GameError::invalid_input(format!("Unknown station type: {:?}", station_type)))?;
        
        Ok(StationInstance {
            id: new_uuid(),
            station_type,
            floor,
            position,
            operated_by: None,
            last_used: 0.0,
            health: 100, // Default health
            max_health: 100,
            upgrade_level: 1,
        })
    }
    
    /// Execute a button press on a station
    pub fn execute_button_action(
        &self,
        station: &mut StationInstance,
        button_index: u8,
        context: &StationActionContext,
    ) -> GameResult<StationActionResult> {
        let definition = self.get_definition(station.station_type)
            .ok_or_else(|| GameError::invalid_input(format!("Unknown station type: {:?}", station.station_type)))?;
        
        // Check if button index is valid
        if button_index >= definition.button_count {
            return Err(GameError::invalid_input(format!(
                "Button index {} out of range for station type {:?}",
                button_index, station.station_type
            )));
        }
        
        let button = definition.button_definitions.get(button_index as usize)
            .ok_or_else(|| GameError::invalid_input("Button definition not found".to_string()))?;
        
        // Check cooldown
        if station.last_used + button.cooldown_seconds > context.current_time {
            return Ok(StationActionResult {
                success: false,
                message: format!("Button on cooldown for {:.1} seconds", 
                    (station.last_used + button.cooldown_seconds) - context.current_time),
                effects: Vec::new(),
                cooldown_applied: 0.0,
            });
        }
        
        // Check resource requirements
        for (resource_type, required_amount) in &button.resource_cost {
            let available = context.available_resources.get(resource_type).unwrap_or(&0);
            if available < required_amount {
                return Ok(StationActionResult {
                    success: false,
                    message: format!("Insufficient {resource_type:?}: need {required_amount}, have {available}"),
                    effects: Vec::new(),
                    cooldown_applied: 0.0,
                });
            }
        }
        
        // Execute the action
        let mut effects = Vec::new();
        let mut success = true;
        let mut message = "Action executed successfully".to_string();
        
        match &button.action {
            StationAction::FireWeapon { weapon_type, damage, range: _, speed } => {
                if let Some(target_id) = context.nearest_enemy {
                    let actual_damage = damage + (station.upgrade_level as u32 - 1) * 10; // Damage scales with upgrade
                    
                    match speed {
                        None => {
                            // Instant weapon (laser)
                            effects.push(StationEffect::Damage {
                                target_id,
                                amount: actual_damage,
                            });
                        }
                        Some(_projectile_speed) => {
                            // Projectile weapon
                            let projectile_id = new_uuid();
                            effects.push(StationEffect::ProjectileCreated {
                                projectile_id,
                                position: context.station_world_pos,
                                velocity: context.direction_to_target.unwrap_or((0.0, 0.0)),
                                damage: actual_damage,
                            });
                        }
                    }
                    
                    message = format!("Fired {:?} at target", weapon_type);
                } else {
                    success = false;
                    message = "No target in range".to_string();
                }
            }
            
            StationAction::BoostShield { amount } => {
                if let Some(mech_id) = context.mech_id {
                    let boost_amount = amount + (station.upgrade_level as u32 - 1) * 5;
                    effects.push(StationEffect::ShieldBoost {
                        target_id: mech_id,
                        amount: boost_amount,
                    });
                    message = format!("Boosted shield by {}", boost_amount);
                }
            }
            
            StationAction::RepairMech { hp_per_resource } => {
                if let Some(mech_id) = context.mech_id {
                    let scrap_available = context.available_resources.get(&ResourceType::ScrapMetal).unwrap_or(&0);
                    if *scrap_available > 0 {
                        let repair_amount = hp_per_resource * *scrap_available;
                        effects.push(StationEffect::Heal {
                            target_id: mech_id,
                            amount: repair_amount,
                        });
                        effects.push(StationEffect::ResourceConsumed {
                            resource_type: ResourceType::ScrapMetal,
                            amount: *scrap_available,
                        });
                        message = format!("Repaired {} HP", repair_amount);
                    } else {
                        success = false;
                        message = "No scrap metal available for repairs".to_string();
                    }
                }
            }
            
            StationAction::UpgradeMech { upgrade_type } => {
                if let Some(mech_id) = context.mech_id {
                    // Check if upgrade is possible
                    let current_level = context.current_upgrade_levels.get(upgrade_type).unwrap_or(&1);
                    if *current_level < 5 { // Max level
                        effects.push(StationEffect::UpgradeApplied {
                            target_id: mech_id,
                            upgrade_type: *upgrade_type,
                            new_level: current_level + 1,
                        });
                        message = format!("Upgraded {:?} to level {}", upgrade_type, current_level + 1);
                    } else {
                        success = false;
                        message = format!("{:?} already at maximum level", upgrade_type);
                    }
                }
            }
            
            StationAction::ChargeEnergy { energy_per_tick } => {
                if let Some(mech_id) = context.mech_id {
                    effects.push(StationEffect::EnergyCharge {
                        target_id: mech_id,
                        amount: *energy_per_tick,
                    });
                    message = format!("Charging energy: +{}", energy_per_tick);
                }
            }
            
            StationAction::TriggerEffect { effect, duration } => {
                if let Some(mech_id) = context.mech_id {
                    effects.push(StationEffect::TemporaryBuff {
                        target_id: mech_id,
                        buff_type: effect.clone(),
                        duration: *duration,
                    });
                    message = format!("Activated {} for {}s", effect, duration);
                }
            }
            
            StationAction::None => {
                success = false;
                message = "No action configured for this button".to_string();
            }
        }
        
        // Consume resources
        for (resource_type, amount) in &button.resource_cost {
            effects.push(StationEffect::ResourceConsumed {
                resource_type: *resource_type,
                amount: *amount,
            });
        }
        
        // Apply cooldown
        if success {
            station.last_used = context.current_time;
        }
        
        Ok(StationActionResult {
            success,
            message,
            effects,
            cooldown_applied: button.cooldown_seconds,
        })
    }
    
    /// Register default station types
    fn register_default_stations(&mut self) {
        // Laser weapon station
        self.register_station(StationDefinition {
            station_type: StationType::WeaponLaser,
            name: "Laser Cannon".to_string(),
            description: "High-precision energy weapon with instant damage".to_string(),
            button_count: 1,
            button_definitions: vec![
                ButtonDefinition {
                    index: 0,
                    label: "Fire".to_string(),
                    description: "Fire laser at nearest enemy".to_string(),
                    action: StationAction::FireWeapon {
                        weapon_type: WeaponType::Laser,
                        damage: 25,
                        range: 50.0,
                        speed: None, // Instant
                    },
                    cooldown_seconds: 2.0,
                    resource_cost: HashMap::new(),
                },
            ],
            cooldown_seconds: 2.0,
            resource_requirements: HashMap::new(),
            upgrade_requirements: HashMap::from([
                (ResourceType::ComputerComponents, 2),
                (ResourceType::Wiring, 1),
            ]),
            allowed_floors: vec![1],
            max_per_mech: 2,
            size: (1, 1),
        });
        
        // Projectile weapon station
        self.register_station(StationDefinition {
            station_type: StationType::WeaponProjectile,
            name: "Projectile Cannon".to_string(),
            description: "Ballistic weapon that fires explosive projectiles".to_string(),
            button_count: 1,
            button_definitions: vec![
                ButtonDefinition {
                    index: 0,
                    label: "Fire".to_string(),
                    description: "Fire projectile at nearest enemy".to_string(),
                    action: StationAction::FireWeapon {
                        weapon_type: WeaponType::Projectile,
                        damage: 35,
                        range: 60.0,
                        speed: Some(300.0),
                    },
                    cooldown_seconds: 3.0,
                    resource_cost: HashMap::new(),
                },
            ],
            cooldown_seconds: 3.0,
            resource_requirements: HashMap::new(),
            upgrade_requirements: HashMap::from([
                (ResourceType::ScrapMetal, 3),
            ]),
            allowed_floors: vec![1],
            max_per_mech: 2,
            size: (1, 1),
        });
        
        // Shield station
        self.register_station(StationDefinition {
            station_type: StationType::Shield,
            name: "Shield Generator".to_string(),
            description: "Defensive system that boosts mech shields".to_string(),
            button_count: 1,
            button_definitions: vec![
                ButtonDefinition {
                    index: 0,
                    label: "Boost".to_string(),
                    description: "Boost mech shield strength".to_string(),
                    action: StationAction::BoostShield {
                        amount: 25,
                    },
                    cooldown_seconds: 5.0,
                    resource_cost: HashMap::from([
                        (ResourceType::Batteries, 1),
                    ]),
                },
            ],
            cooldown_seconds: 5.0,
            resource_requirements: HashMap::new(),
            upgrade_requirements: HashMap::from([
                (ResourceType::Batteries, 2),
                (ResourceType::Wiring, 1),
            ]),
            allowed_floors: vec![1],
            max_per_mech: 1,
            size: (1, 1),
        });
        
        // Engine station
        self.register_station(StationDefinition {
            station_type: StationType::Engine,
            name: "Engine Control".to_string(),
            description: "Controls mech movement and propulsion".to_string(),
            button_count: 0, // Uses WASD controls, not buttons
            button_definitions: vec![],
            cooldown_seconds: 0.0,
            resource_requirements: HashMap::new(),
            upgrade_requirements: HashMap::from([
                (ResourceType::ComputerComponents, 2),
                (ResourceType::Wiring, 2),
            ]),
            allowed_floors: vec![0],
            max_per_mech: 1,
            size: (1, 1),
        });
        
        // Repair station
        self.register_station(StationDefinition {
            station_type: StationType::Repair,
            name: "Repair Bay".to_string(),
            description: "Restores mech hull integrity using scrap metal".to_string(),
            button_count: 1,
            button_definitions: vec![
                ButtonDefinition {
                    index: 0,
                    label: "Repair".to_string(),
                    description: "Repair mech hull damage".to_string(),
                    action: StationAction::RepairMech {
                        hp_per_resource: 20,
                    },
                    cooldown_seconds: 1.0,
                    resource_cost: HashMap::new(), // Uses available scrap metal
                },
            ],
            cooldown_seconds: 1.0,
            resource_requirements: HashMap::new(),
            upgrade_requirements: HashMap::from([
                (ResourceType::ScrapMetal, 2),
            ]),
            allowed_floors: vec![2],
            max_per_mech: 1,
            size: (1, 1),
        });
        
        // Upgrade station
        self.register_station(StationDefinition {
            station_type: StationType::Upgrade,
            name: "Upgrade Terminal".to_string(),
            description: "Enhances mech systems and capabilities".to_string(),
            button_count: 4,
            button_definitions: vec![
                ButtonDefinition {
                    index: 0,
                    label: "Upgrade Laser".to_string(),
                    description: "Improve laser weapon damage and efficiency".to_string(),
                    action: StationAction::UpgradeMech {
                        upgrade_type: MechUpgradeType::Laser,
                    },
                    cooldown_seconds: 1.0,
                    resource_cost: HashMap::from([
                        (ResourceType::ScrapMetal, 2),
                        (ResourceType::ComputerComponents, 1),
                    ]),
                },
                ButtonDefinition {
                    index: 1,
                    label: "Upgrade Projectile".to_string(),
                    description: "Improve projectile weapon damage and speed".to_string(),
                    action: StationAction::UpgradeMech {
                        upgrade_type: MechUpgradeType::Projectile,
                    },
                    cooldown_seconds: 1.0,
                    resource_cost: HashMap::from([
                        (ResourceType::ScrapMetal, 3),
                    ]),
                },
                ButtonDefinition {
                    index: 2,
                    label: "Upgrade Shield".to_string(),
                    description: "Improve shield capacity and recharge rate".to_string(),
                    action: StationAction::UpgradeMech {
                        upgrade_type: MechUpgradeType::Shield,
                    },
                    cooldown_seconds: 1.0,
                    resource_cost: HashMap::from([
                        (ResourceType::Batteries, 2),
                        (ResourceType::Wiring, 1),
                    ]),
                },
                ButtonDefinition {
                    index: 3,
                    label: "Upgrade Engine".to_string(),
                    description: "Improve mech speed and maneuverability".to_string(),
                    action: StationAction::UpgradeMech {
                        upgrade_type: MechUpgradeType::Engine,
                    },
                    cooldown_seconds: 1.0,
                    resource_cost: HashMap::from([
                        (ResourceType::ComputerComponents, 2),
                        (ResourceType::Wiring, 2),
                    ]),
                },
            ],
            cooldown_seconds: 1.0,
            resource_requirements: HashMap::new(),
            upgrade_requirements: HashMap::new(),
            allowed_floors: vec![0],
            max_per_mech: 1,
            size: (1, 1),
        });
        
        // Electrical station
        self.register_station(StationDefinition {
            station_type: StationType::Electrical,
            name: "Power Management".to_string(),
            description: "Manages mech power systems and energy distribution".to_string(),
            button_count: 2,
            button_definitions: vec![
                ButtonDefinition {
                    index: 0,
                    label: "Recharge".to_string(),
                    description: "Recharge energy reserves".to_string(),
                    action: StationAction::ChargeEnergy {
                        energy_per_tick: 10,
                    },
                    cooldown_seconds: 0.5,
                    resource_cost: HashMap::from([
                        (ResourceType::Batteries, 1),
                    ]),
                },
                ButtonDefinition {
                    index: 1,
                    label: "Boost Systems".to_string(),
                    description: "Temporarily boost all systems".to_string(),
                    action: StationAction::TriggerEffect { 
                        effect: "EnergyBoost".to_string(),
                        duration: 10.0,
                    },
                    cooldown_seconds: 30.0,
                    resource_cost: HashMap::from([
                        (ResourceType::Batteries, 2),
                        (ResourceType::Wiring, 1),
                    ]),
                },
            ],
            cooldown_seconds: 0.5,
            resource_requirements: HashMap::new(),
            upgrade_requirements: HashMap::from([
                (ResourceType::Wiring, 2),
                (ResourceType::Batteries, 1),
            ]),
            allowed_floors: vec![1, 2],
            max_per_mech: 2,
            size: (1, 1),
        });
        
        // Pilot station
        self.register_station(StationDefinition {
            station_type: StationType::Pilot,
            name: "Pilot Control".to_string(),
            description: "Command center for controlling mech movement and systems".to_string(),
            button_count: 0, // Uses WASD controls and special UI
            button_definitions: vec![],
            cooldown_seconds: 0.0,
            resource_requirements: HashMap::new(),
            upgrade_requirements: HashMap::from([
                (ResourceType::ComputerComponents, 3),
                (ResourceType::Wiring, 2),
            ]),
            allowed_floors: vec![2], // Top floor only
            max_per_mech: 1,
            size: (1, 1),
        });
    }
}

/// Context information needed for station actions
#[derive(Debug, Clone)]
pub struct StationActionContext {
    pub current_time: f32,
    pub mech_id: Option<Uuid>,
    pub station_world_pos: crate::WorldPos,
    pub available_resources: HashMap<ResourceType, u32>,
    pub nearest_enemy: Option<Uuid>,
    pub direction_to_target: Option<(f32, f32)>,
    pub current_upgrade_levels: HashMap<MechUpgradeType, u8>,
}

impl Default for StationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl StationInstance {
    /// Check if this station can be operated
    pub fn can_operate(&self, _current_time: f32) -> bool {
        self.operated_by.is_none() && self.health > 0
    }
    
    /// Check if this station is on cooldown
    pub fn is_on_cooldown(&self, current_time: f32, cooldown_duration: f32) -> bool {
        self.last_used + cooldown_duration > current_time
    }
    
    /// Get remaining cooldown time
    pub fn remaining_cooldown(&self, current_time: f32, cooldown_duration: f32) -> f32 {
        let remaining = (self.last_used + cooldown_duration) - current_time;
        remaining.max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_station_registry_creation() {
        let registry = StationRegistry::new();
        assert!(registry.get_definition(StationType::WeaponLaser).is_some());
        assert!(registry.get_definition(StationType::Shield).is_some());
        assert!(registry.get_definition(StationType::Engine).is_some());
    }
    
    #[test]
    fn test_station_creation() {
        let registry = StationRegistry::new();
        let station = registry.create_station(
            StationType::WeaponLaser,
            1,
            TilePos::new(5, 5),
        ).unwrap();
        
        assert_eq!(station.station_type, StationType::WeaponLaser);
        assert_eq!(station.floor, 1);
        assert_eq!(station.position, TilePos::new(5, 5));
        assert!(station.can_operate(0.0));
    }
    
    #[test]
    fn test_button_action_execution() {
        let registry = StationRegistry::new();
        let mut station = registry.create_station(
            StationType::Shield,
            1,
            TilePos::new(5, 5),
        ).unwrap();
        
        let context = StationActionContext {
            current_time: 0.0,
            mech_id: Some(new_uuid()),
            station_world_pos: crate::WorldPos::new(160.0, 160.0),
            available_resources: HashMap::from([
                (ResourceType::Batteries, 5),
            ]),
            nearest_enemy: None,
            direction_to_target: None,
            current_upgrade_levels: HashMap::new(),
        };
        
        let result = registry.execute_button_action(&mut station, 0, &context).unwrap();
        assert!(result.success);
        assert_eq!(result.effects.len(), 2); // ShieldBoost + ResourceConsumed
    }
}