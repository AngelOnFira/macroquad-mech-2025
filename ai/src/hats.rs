use shared::*;
use crate::{Perception, HealthStatus};
use std::collections::HashMap;

/// A "hat" represents a role or mindset the AI can adopt
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Hat {
    // Primary hats
    Pilot,
    Gunner,
    Engineer,
    Scavenger,
    Scout,
    Defender,
    
    // Reactive hats (triggered by events)
    UnderAttack,
    EmergencyRepair,
    ResourceRush,
    Retreating,
    Pursuing,
    
    // Special hats
    Captain,
    Support,
    Idle,
}

impl Hat {
    /// Get display name for the hat
    pub fn name(&self) -> &'static str {
        match self {
            Hat::Pilot => "Pilot",
            Hat::Gunner => "Gunner",
            Hat::Engineer => "Engineer",
            Hat::Scavenger => "Scavenger",
            Hat::Scout => "Scout",
            Hat::Defender => "Defender",
            Hat::UnderAttack => "Under Attack",
            Hat::EmergencyRepair => "Emergency Repair",
            Hat::ResourceRush => "Resource Rush",
            Hat::Retreating => "Retreating",
            Hat::Pursuing => "Pursuing",
            Hat::Captain => "Captain",
            Hat::Support => "Support",
            Hat::Idle => "Idle",
        }
    }
    
    /// Check if this is a reactive hat
    pub fn is_reactive(&self) -> bool {
        matches!(self, 
            Hat::UnderAttack | 
            Hat::EmergencyRepair | 
            Hat::ResourceRush | 
            Hat::Retreating | 
            Hat::Pursuing
        )
    }
    
    /// Get priority level (higher = more important)
    pub fn priority(&self) -> u8 {
        match self {
            Hat::UnderAttack => 10,
            Hat::EmergencyRepair => 9,
            Hat::Retreating => 8,
            Hat::Captain => 7,
            Hat::Pilot => 6,
            Hat::Gunner => 5,
            Hat::ResourceRush => 5,
            Hat::Pursuing => 4,
            Hat::Engineer => 3,
            Hat::Defender => 3,
            Hat::Scavenger => 2,
            Hat::Scout => 2,
            Hat::Support => 1,
            Hat::Idle => 0,
        }
    }
}

/// Task that can be performed while wearing a hat
#[derive(Debug, Clone)]
pub struct Task {
    pub name: String,
    pub priority: f32,
    pub action: TaskAction,
    pub requirements: TaskRequirements,
}

/// Specific action to take
#[derive(Debug, Clone)]
pub enum TaskAction {
    MoveToPosition { target: WorldPos, reason: String },
    OperateStation { station_type: StationType },
    CollectResource { resource_type: Option<ResourceType> },
    FollowPlayer { player_id: uuid::Uuid },
    AttackTarget { target_id: uuid::Uuid },
    DefendPosition { position: WorldPos },
    RepairMech,
    Idle,
}

/// Requirements for a task to be valid
#[derive(Debug, Clone, Default)]
pub struct TaskRequirements {
    pub location: Option<LocationRequirement>,
    pub carrying: Option<CarryingRequirement>,
    pub not_operating: bool,
    pub team_needs: Vec<ResourceType>,
}

#[derive(Debug, Clone)]
pub enum LocationRequirement {
    Outside,
    InsideMech,
    InsideSpecificMech(uuid::Uuid),
    NearPosition(WorldPos, f32),
}

#[derive(Debug, Clone)]
pub enum CarryingRequirement {
    Nothing,
    Resource(Option<ResourceType>),
}

/// Manager for the hat system
pub struct HatManager {
    current_hat: Hat,
    reactive_hat: Option<Hat>,
    available_tasks: HashMap<Hat, Vec<Task>>,
    hat_scores: HashMap<Hat, f32>,
}

impl HatManager {
    pub fn new() -> Self {
        let mut manager = Self {
            current_hat: Hat::Idle,
            reactive_hat: None,
            available_tasks: HashMap::new(),
            hat_scores: HashMap::new(),
        };
        
        manager.initialize_tasks();
        manager
    }
    
    /// Get current active hat (reactive takes precedence)
    pub fn get_active_hat(&self) -> Hat {
        self.reactive_hat.unwrap_or(self.current_hat)
    }
    
    /// Update hat based on perception
    pub fn update_hat(&mut self, perception: &Perception) {
        // Check for reactive hats first
        let old_reactive = self.reactive_hat;
        self.reactive_hat = self.check_reactive_conditions(perception);
        
        if self.reactive_hat.is_some() && old_reactive != self.reactive_hat {
            log::debug!("Switching to reactive hat: {:?} -> {:?}", old_reactive, self.reactive_hat);
        }
        
        // If no reactive hat, evaluate primary hats
        if self.reactive_hat.is_none() {
            self.evaluate_hat_scores(perception);
            
            // Switch hat if a better one is available
            if let Some((best_hat, score)) = self.hat_scores.iter()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap()) 
            {
                if *score > self.hat_scores.get(&self.current_hat).unwrap_or(&0.0) + 0.2 {
                    log::debug!("Switching primary hat: {:?} -> {:?} (score: {})", self.current_hat, best_hat, score);
                    self.current_hat = *best_hat;
                }
            }
        }
    }
    
    /// Get tasks for current hat
    pub fn get_current_tasks(&self, perception: &Perception) -> Vec<Task> {
        let active_hat = self.get_active_hat();
        
        self.available_tasks.get(&active_hat)
            .map(|tasks| {
                tasks.iter()
                    .filter(|task| self.task_is_valid(task, perception))
                    .cloned()
                    .collect()
            })
            .unwrap_or_else(Vec::new)
    }
    
    /// Select a task from available tasks
    pub fn select_task(&self, tasks: &[Task], perception: &Perception) -> Option<Task> {
        if tasks.is_empty() {
            return None;
        }
        
        // Sort by priority and select
        let mut sorted_tasks = tasks.to_vec();
        sorted_tasks.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());
        
        // Sometimes pick a random task for variety (based on difficulty)
        // Use deterministic selection based on AI ID
        let pseudo_random = (perception.my_id.as_u128() as f32 % 100.0) / 100.0;
        if pseudo_random < 0.2 { // 20% chance of random selection
            let index = (perception.my_id.as_u128() as usize % 3) % sorted_tasks.len().min(3).max(1);
            Some(sorted_tasks[index].clone())
        } else {
            sorted_tasks.first().cloned()
        }
    }
    
    /// Check if reactive conditions are met
    fn check_reactive_conditions(&self, perception: &Perception) -> Option<Hat> {
        // Under attack if threats are severe
        if perception.threats.iter().any(|t| t.severity > 0.7) {
            return Some(Hat::UnderAttack);
        }
        
        // Emergency repair if mech health is critical
        if perception.team_state.mech_health.values()
            .any(|(health, _)| *health < 30) {
            return Some(Hat::EmergencyRepair);
        }
        
        // Resource rush if critically low (temporarily lowered threshold for testing)
        if perception.team_state.resource_status.scarcity_level > 0.99 {
            log::debug!("ResourceRush triggered: scarcity_level = {}", perception.team_state.resource_status.scarcity_level);
            return Some(Hat::ResourceRush);
        }
        
        // Retreating if health is low and threats exist
        if matches!(perception.my_state.health_status, HealthStatus::Critical | HealthStatus::Damaged) 
            && !perception.threats.is_empty() {
            return Some(Hat::Retreating);
        }
        
        None
    }
    
    /// Evaluate scores for each hat
    fn evaluate_hat_scores(&mut self, perception: &Perception) {
        self.hat_scores.clear();
        
        // Pilot - valuable if no one is piloting
        let pilot_score = if perception.my_state.operating_station == Some(StationType::Engine) {
            1.0
        } else if perception.team_state.player_roles.values()
            .filter(|role| *role == "Pilot").count() == 0 {
            0.8
        } else {
            0.2
        };
        self.hat_scores.insert(Hat::Pilot, pilot_score);
        
        // Gunner - valuable in combat
        let gunner_score = if !perception.threats.is_empty() {
            0.9
        } else if perception.team_state.combat_readiness > 0.7 {
            0.6
        } else {
            0.3
        };
        self.hat_scores.insert(Hat::Gunner, gunner_score);
        
        // Scavenger - valuable when resources are low
        let scavenger_score = perception.team_state.resource_status.scarcity_level;
        self.hat_scores.insert(Hat::Scavenger, scavenger_score);
        
        // Engineer - valuable when repairs/upgrades needed
        let engineer_score = if perception.team_state.mech_health.values()
            .any(|(health, _)| *health < 70) {
            0.7
        } else {
            0.3
        };
        self.hat_scores.insert(Hat::Engineer, engineer_score);
        
        // Scout - valuable when no threats and resources unknown
        let scout_score = if perception.threats.is_empty() && perception.environment.nearby_resources.len() < 3 {
            0.5
        } else {
            0.2
        };
        self.hat_scores.insert(Hat::Scout, scout_score);
        
        // Defender - valuable when defending
        let defender_score = if perception.team_state.combat_readiness < 0.5 {
            0.6
        } else {
            0.3
        };
        self.hat_scores.insert(Hat::Defender, defender_score);
    }
    
    /// Check if a task is valid given current perception
    fn task_is_valid(&self, task: &Task, perception: &Perception) -> bool {
        // Check location requirement
        if let Some(loc_req) = &task.requirements.location {
            match loc_req {
                LocationRequirement::Outside => {
                    if !matches!(perception.my_state.location, PlayerLocation::OutsideWorld(_)) {
                        return false;
                    }
                }
                LocationRequirement::InsideMech => {
                    if !matches!(perception.my_state.location, PlayerLocation::InsideMech { .. }) {
                        return false;
                    }
                }
                LocationRequirement::InsideSpecificMech(mech_id) => {
                    if let PlayerLocation::InsideMech { mech_id: my_mech, .. } = perception.my_state.location {
                        if my_mech != *mech_id {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                LocationRequirement::NearPosition(pos, max_dist) => {
                    let my_pos = match perception.my_state.location {
                        PlayerLocation::OutsideWorld(p) => p,
                        PlayerLocation::InsideMech { .. } => return false,
                    };
                    if my_pos.distance_to(*pos) > *max_dist {
                        return false;
                    }
                }
            }
        }
        
        // Check carrying requirement
        if let Some(carry_req) = &task.requirements.carrying {
            match carry_req {
                CarryingRequirement::Nothing => {
                    if perception.my_state.carrying_resource.is_some() {
                        return false;
                    }
                }
                CarryingRequirement::Resource(specific) => {
                    if let Some(res_type) = specific {
                        if perception.my_state.carrying_resource != Some(*res_type) {
                            return false;
                        }
                    } else if perception.my_state.carrying_resource.is_none() {
                        return false;
                    }
                }
            }
        }
        
        // Check not operating requirement
        if task.requirements.not_operating && perception.my_state.operating_station.is_some() {
            return false;
        }
        
        true
    }
    
    /// Initialize tasks for each hat
    fn initialize_tasks(&mut self) {
        // Pilot tasks
        self.available_tasks.insert(Hat::Pilot, vec![
            Task {
                name: "Operate Engine".to_string(),
                priority: 1.0,
                action: TaskAction::OperateStation { station_type: StationType::Engine },
                requirements: TaskRequirements {
                    location: Some(LocationRequirement::InsideMech),
                    not_operating: true,
                    ..Default::default()
                },
            },
            Task {
                name: "Move to Engine".to_string(),
                priority: 0.8,
                action: TaskAction::MoveToPosition { 
                    target: WorldPos::new(0.0, 0.0), // Will be calculated dynamically
                    reason: "Getting to engine station".to_string(),
                },
                requirements: TaskRequirements {
                    location: Some(LocationRequirement::Outside),
                    ..Default::default()
                },
            },
        ]);
        
        // Gunner tasks
        self.available_tasks.insert(Hat::Gunner, vec![
            Task {
                name: "Operate Laser".to_string(),
                priority: 0.9,
                action: TaskAction::OperateStation { station_type: StationType::WeaponLaser },
                requirements: TaskRequirements {
                    location: Some(LocationRequirement::InsideMech),
                    not_operating: true,
                    ..Default::default()
                },
            },
            Task {
                name: "Operate Projectile".to_string(),
                priority: 0.8,
                action: TaskAction::OperateStation { station_type: StationType::WeaponProjectile },
                requirements: TaskRequirements {
                    location: Some(LocationRequirement::InsideMech),
                    not_operating: true,
                    ..Default::default()
                },
            },
        ]);
        
        // Scavenger tasks
        self.available_tasks.insert(Hat::Scavenger, vec![
            Task {
                name: "Collect Any Resource".to_string(),
                priority: 0.7,
                action: TaskAction::CollectResource { resource_type: None },
                requirements: TaskRequirements {
                    location: Some(LocationRequirement::Outside),
                    carrying: Some(CarryingRequirement::Nothing),
                    ..Default::default()
                },
            },
            Task {
                name: "Deliver Resource".to_string(),
                priority: 0.9,
                action: TaskAction::MoveToPosition {
                    target: WorldPos::new(0.0, 0.0), // Will be calculated to mech position
                    reason: "Delivering resource".to_string(),
                },
                requirements: TaskRequirements {
                    carrying: Some(CarryingRequirement::Resource(None)),
                    ..Default::default()
                },
            },
        ]);
        
        // Engineer tasks
        self.available_tasks.insert(Hat::Engineer, vec![
            Task {
                name: "Operate Repair".to_string(),
                priority: 0.8,
                action: TaskAction::OperateStation { station_type: StationType::Repair },
                requirements: TaskRequirements {
                    location: Some(LocationRequirement::InsideMech),
                    not_operating: true,
                    ..Default::default()
                },
            },
            Task {
                name: "Operate Upgrade".to_string(),
                priority: 0.6,
                action: TaskAction::OperateStation { station_type: StationType::Upgrade },
                requirements: TaskRequirements {
                    location: Some(LocationRequirement::InsideMech),
                    not_operating: true,
                    ..Default::default()
                },
            },
        ]);
        
        // Scout tasks
        self.available_tasks.insert(Hat::Scout, vec![
            Task {
                name: "Explore Area".to_string(),
                priority: 0.5,
                action: TaskAction::MoveToPosition {
                    target: WorldPos::new(0.0, 0.0), // Will be calculated
                    reason: "Exploring for resources".to_string(),
                },
                requirements: TaskRequirements {
                    location: Some(LocationRequirement::Outside),
                    ..Default::default()
                },
            },
        ]);
        
        // Under Attack tasks
        self.available_tasks.insert(Hat::UnderAttack, vec![
            Task {
                name: "Retreat to Safety".to_string(),
                priority: 1.0,
                action: TaskAction::MoveToPosition {
                    target: WorldPos::new(0.0, 0.0), // Will be nearest safe location
                    reason: "Escaping danger".to_string(),
                },
                requirements: TaskRequirements {
                    location: Some(LocationRequirement::Outside),
                    ..Default::default()
                },
            },
            Task {
                name: "Fight Back".to_string(),
                priority: 0.7,
                action: TaskAction::OperateStation { station_type: StationType::WeaponLaser },
                requirements: TaskRequirements {
                    location: Some(LocationRequirement::InsideMech),
                    ..Default::default()
                },
            },
        ]);
        
        // Emergency Repair tasks
        self.available_tasks.insert(Hat::EmergencyRepair, vec![
            Task {
                name: "Emergency Repair".to_string(),
                priority: 1.0,
                action: TaskAction::OperateStation { station_type: StationType::Repair },
                requirements: TaskRequirements {
                    location: Some(LocationRequirement::InsideMech),
                    not_operating: true,
                    ..Default::default()
                },
            },
            Task {
                name: "Rush to Repair Station".to_string(),
                priority: 0.9,
                action: TaskAction::MoveToPosition {
                    target: WorldPos::new(0.0, 0.0), // Will be calculated to repair station
                    reason: "Getting to repair station urgently".to_string(),
                },
                requirements: TaskRequirements {
                    location: Some(LocationRequirement::Outside),
                    ..Default::default()
                },
            },
        ]);
        
        // Resource Rush tasks
        self.available_tasks.insert(Hat::ResourceRush, vec![
            Task {
                name: "Collect Critical Resource".to_string(),
                priority: 0.9,
                action: TaskAction::CollectResource { resource_type: None },
                requirements: TaskRequirements {
                    location: Some(LocationRequirement::Outside),
                    carrying: Some(CarryingRequirement::Nothing),
                    ..Default::default()
                },
            },
            Task {
                name: "Rush Deliver Resource".to_string(),
                priority: 1.0,
                action: TaskAction::MoveToPosition {
                    target: WorldPos::new(0.0, 0.0), // Will be calculated to mech position
                    reason: "Urgently delivering resource".to_string(),
                },
                requirements: TaskRequirements {
                    carrying: Some(CarryingRequirement::Resource(None)),
                    ..Default::default()
                },
            },
        ]);
        
        // Retreating tasks
        self.available_tasks.insert(Hat::Retreating, vec![
            Task {
                name: "Retreat to Safe Distance".to_string(),
                priority: 1.0,
                action: TaskAction::MoveToPosition {
                    target: WorldPos::new(0.0, 0.0), // Will be calculated away from threats
                    reason: "Retreating from danger".to_string(),
                },
                requirements: TaskRequirements {
                    location: Some(LocationRequirement::Outside),
                    ..Default::default()
                },
            },
        ]);
        
        // Pursuing tasks
        self.available_tasks.insert(Hat::Pursuing, vec![
            Task {
                name: "Chase Enemy".to_string(),
                priority: 0.8,
                action: TaskAction::MoveToPosition {
                    target: WorldPos::new(0.0, 0.0), // Will be calculated to enemy position
                    reason: "Pursuing enemy".to_string(),
                },
                requirements: TaskRequirements {
                    location: Some(LocationRequirement::Outside),
                    ..Default::default()
                },
            },
            Task {
                name: "Fire at Enemy".to_string(),
                priority: 0.9,
                action: TaskAction::OperateStation { station_type: StationType::WeaponLaser },
                requirements: TaskRequirements {
                    location: Some(LocationRequirement::InsideMech),
                    ..Default::default()
                },
            },
        ]);
        
        // Defender tasks
        self.available_tasks.insert(Hat::Defender, vec![
            Task {
                name: "Activate Shield".to_string(),
                priority: 0.8,
                action: TaskAction::OperateStation { station_type: StationType::Shield },
                requirements: TaskRequirements {
                    location: Some(LocationRequirement::InsideMech),
                    not_operating: true,
                    ..Default::default()
                },
            },
            Task {
                name: "Defend Position".to_string(),
                priority: 0.6,
                action: TaskAction::MoveToPosition {
                    target: WorldPos::new(0.0, 0.0), // Will be calculated to defensive position
                    reason: "Moving to defensive position".to_string(),
                },
                requirements: TaskRequirements {
                    location: Some(LocationRequirement::Outside),
                    ..Default::default()
                },
            },
        ]);
        
        // Captain tasks
        self.available_tasks.insert(Hat::Captain, vec![
            Task {
                name: "Coordinate Team".to_string(),
                priority: 0.5,
                action: TaskAction::MoveToPosition {
                    target: WorldPos::new(0.0, 0.0), // Will be calculated to central position
                    reason: "Coordinating team from central position".to_string(),
                },
                requirements: TaskRequirements {
                    location: Some(LocationRequirement::Outside),
                    ..Default::default()
                },
            },
            Task {
                name: "Pilot Mech".to_string(),
                priority: 0.7,
                action: TaskAction::OperateStation { station_type: StationType::Engine },
                requirements: TaskRequirements {
                    location: Some(LocationRequirement::InsideMech),
                    not_operating: true,
                    ..Default::default()
                },
            },
        ]);
        
        // Support tasks
        self.available_tasks.insert(Hat::Support, vec![
            Task {
                name: "Assist Teammate".to_string(),
                priority: 0.6,
                action: TaskAction::MoveToPosition {
                    target: WorldPos::new(0.0, 0.0), // Will be calculated to teammate position
                    reason: "Moving to assist teammate".to_string(),
                },
                requirements: TaskRequirements {
                    location: Some(LocationRequirement::Outside),
                    ..Default::default()
                },
            },
            Task {
                name: "Provide Backup".to_string(),
                priority: 0.5,
                action: TaskAction::OperateStation { station_type: StationType::WeaponLaser },
                requirements: TaskRequirements {
                    location: Some(LocationRequirement::InsideMech),
                    ..Default::default()
                },
            },
        ]);
        
        // Idle tasks
        self.available_tasks.insert(Hat::Idle, vec![
            Task {
                name: "Patrol Area".to_string(),
                priority: 0.3,
                action: TaskAction::MoveToPosition {
                    target: WorldPos::new(0.0, 0.0), // Will be calculated to patrol point
                    reason: "Patrolling area".to_string(),
                },
                requirements: TaskRequirements {
                    location: Some(LocationRequirement::Outside),
                    ..Default::default()
                },
            },
            Task {
                name: "Stand By".to_string(),
                priority: 0.1,
                action: TaskAction::MoveToPosition {
                    target: WorldPos::new(0.0, 0.0), // Will be calculated to safe position
                    reason: "Standing by for orders".to_string(),
                },
                requirements: TaskRequirements {
                    ..Default::default()
                },
            },
        ]);
    }
}

impl Default for HatManager {
    fn default() -> Self {
        Self::new()
    }
}