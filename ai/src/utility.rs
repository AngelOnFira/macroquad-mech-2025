use crate::{
    AIController, AIDebugInfo, AIMessage, Decision, GameView, HatManager, IntelInfo, Perception,
    Personality, Status, Task, TaskAction,
};
use shared::*;
use std::collections::HashMap;
use uuid::Uuid;

/// Utility-based AI that scores actions and picks the best one
pub struct UtilityAI {
    id: Uuid,
    personality: Personality,
    difficulty: f32,
    hat_manager: HatManager,
    last_decision: Option<Decision>,
    decision_history: Vec<(String, f32)>, // (action_name, score)
    state: AIState,
}

/// Simple AI for easier difficulties
pub struct SimpleAI {
    id: Uuid,
    personality: Personality,
    difficulty: f32,
    hat_manager: HatManager,
    last_decision: Option<Decision>,
    state: AIState,
}

/// Internal AI state
#[derive(Debug, Clone)]
struct AIState {
    current_goal: Option<String>,
    stuck_counter: u32,
    last_position: Option<WorldPos>,
    known_resources: HashMap<Uuid, (WorldPos, ResourceType)>,
    recent_threats: Vec<(Uuid, f32)>, // (threat_id, last_seen_time)
}

impl UtilityAI {
    pub fn new(id: Uuid, personality: Personality, difficulty: f32) -> Self {
        Self {
            id,
            personality,
            difficulty,
            hat_manager: HatManager::new(),
            last_decision: None,
            decision_history: Vec::new(),
            state: AIState {
                current_goal: None,
                stuck_counter: 0,
                last_position: None,
                known_resources: HashMap::new(),
                recent_threats: Vec::new(),
            },
        }
    }

    /// Calculate utility score for a task
    fn calculate_utility(&self, task: &Task, perception: &Perception) -> f32 {
        let mut score = task.priority;

        // Adjust based on personality
        score *= self.personality.task_preference(&task.action);

        // Adjust based on current situation
        match &task.action {
            TaskAction::MoveToPosition { target, .. } => {
                // Penalize if we're stuck
                if self.state.stuck_counter > 3 {
                    score *= 0.5;
                }

                // Consider distance
                if let Some(my_pos) = self.get_my_position(perception) {
                    let distance = my_pos.distance_to(*target);
                    score *= 1.0 / (1.0 + distance / 100.0);
                }
            }

            TaskAction::OperateStation { station_type } => {
                // Bonus if we're already at a station
                if perception.my_state.operating_station.is_some() {
                    score *= 1.5;
                }

                // Consider team needs
                score *= self.evaluate_station_need(*station_type, perception);
            }

            TaskAction::CollectResource { resource_type } => {
                // Consider resource scarcity
                score *= perception.team_state.resource_status.scarcity_level;

                // Bonus if we know where resources are
                if resource_type.is_none()
                    || self
                        .state
                        .known_resources
                        .values()
                        .any(|(_, t)| resource_type.map(|rt| rt == *t).unwrap_or(true))
                {
                    score *= 1.2;
                }
            }

            TaskAction::AttackTarget { .. } => {
                // Consider combat readiness
                score *= perception.team_state.combat_readiness;

                // Personality adjustment
                if matches!(self.personality, Personality::Aggressive) {
                    score *= 1.5;
                }
            }

            _ => {}
        }

        // Apply difficulty modifier (higher difficulty = better decisions)
        score *= 0.5 + (self.difficulty * 0.5);

        score
    }

    /// Get current position
    fn get_my_position(&self, perception: &Perception) -> Option<WorldPos> {
        match perception.my_state.location {
            PlayerLocation::OutsideWorld(pos) => Some(pos),
            PlayerLocation::InsideMech { .. } => None,
        }
    }

    /// Evaluate how much a station is needed
    fn evaluate_station_need(&self, station_type: StationType, perception: &Perception) -> f32 {
        match station_type {
            StationType::Engine => {
                // Critical if no one is piloting
                if perception
                    .team_state
                    .player_roles
                    .values()
                    .filter(|r| *r == "Pilot")
                    .count()
                    == 0
                {
                    2.0
                } else {
                    0.5
                }
            }
            StationType::WeaponLaser | StationType::WeaponProjectile => {
                // Important if threats exist
                if !perception.threats.is_empty() {
                    1.5
                } else {
                    0.7
                }
            }
            StationType::Shield => {
                // Important if under attack
                if perception.threats.iter().any(|t| t.severity > 0.5) {
                    1.3
                } else {
                    0.6
                }
            }
            StationType::Repair => {
                // Critical if health is low
                let avg_health = perception
                    .team_state
                    .mech_health
                    .values()
                    .map(|(h, _)| *h as f32 / 100.0)
                    .sum::<f32>()
                    / perception.team_state.mech_health.len().max(1) as f32;
                2.0 - avg_health
            }
            _ => 1.0,
        }
    }

    /// Update internal state
    fn update_state(&mut self, perception: &Perception) {
        // Check if we're stuck
        if let Some(pos) = self.get_my_position(perception) {
            if let Some(last_pos) = self.state.last_position {
                if pos.distance_to(last_pos) < 1.0 {
                    self.state.stuck_counter += 1;
                } else {
                    self.state.stuck_counter = 0;
                }
            }
            self.state.last_position = Some(pos);
        }

        // Update known resources
        for resource in &perception.environment.nearby_resources {
            // Add to known resources (would need resource ID in real implementation)
            let fake_id = Uuid::new_v4();
            self.state.known_resources.insert(fake_id, *resource);
        }

        // Update recent threats
        self.state.recent_threats.retain(|(_, time)| *time > 0.0);
        for threat in &perception.threats {
            if let crate::ThreatType::EnemyMech { id, .. } = &threat.threat_type {
                self.state.recent_threats.push((*id, 5.0)); // Remember for 5 seconds
            }
        }
    }

    /// Generate messages based on perception
    fn generate_messages(&self, perception: &Perception) -> Vec<AIMessage> {
        let mut messages = Vec::new();

        // Report threats
        for threat in &perception.threats {
            if threat.severity > 0.6 {
                if let crate::ThreatType::EnemyMech { .. } = &threat.threat_type {
                    messages.push(AIMessage::intel(
                        self.id,
                        IntelInfo::EnemySpotted {
                            position: threat.position,
                            enemy_type: "Mech".to_string(),
                        },
                    ));
                }
            }
        }

        // Report found resources
        for (pos, resource_type) in &perception.environment.nearby_resources {
            messages.push(AIMessage::intel(
                self.id,
                IntelInfo::ResourceLocation {
                    position: *pos,
                    resource_type: *resource_type,
                },
            ));
        }

        // Report hat changes
        let current_hat = self.hat_manager.get_active_hat();
        messages.push(AIMessage::status(
            self.id,
            Status::ChangingHat {
                new_hat: current_hat.name().to_string(),
            },
        ));

        messages
    }
}

impl AIController for UtilityAI {
    fn id(&self) -> Uuid {
        self.id
    }

    fn perceive(&self, game_view: &GameView) -> Perception {
        Perception::from_game_view(game_view, self.id)
    }

    fn decide(
        &mut self,
        perception: &Perception,
        messages: &[AIMessage],
        delta_time: f32,
    ) -> Decision {
        // Update internal state
        self.update_state(perception);

        // Update hat based on perception
        self.hat_manager.update_hat(perception);

        // Get available tasks for current hat
        let tasks = self.hat_manager.get_current_tasks(perception);

        // Score each task
        let mut scored_tasks: Vec<(Task, f32)> = tasks
            .into_iter()
            .map(|task| {
                let score = self.calculate_utility(&task, perception);
                (task, score)
            })
            .collect();

        // Sort by score
        scored_tasks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Record decision history for debugging
        self.decision_history.clear();
        for (task, score) in &scored_tasks {
            self.decision_history.push((task.name.clone(), *score));
        }

        // Select best task
        let selected_task = scored_tasks.into_iter().next().map(|(task, _)| task);

        // Generate messages
        let messages = self.generate_messages(perception);

        // Create decision
        let decision = Decision {
            chosen_action: selected_task.map(|t| t.action),
            confidence: self.difficulty,
            reasoning: format!(
                "Hat: {}, Goal: {:?}",
                self.hat_manager.get_active_hat().name(),
                self.state.current_goal
            ),
            messages,
        };

        self.last_decision = Some(decision.clone());
        decision
    }

    fn get_debug_info(&self) -> AIDebugInfo {
        AIDebugInfo {
            ai_id: self.id,
            current_hat: self.hat_manager.get_active_hat().name().to_string(),
            personality: format!("{:?}", self.personality),
            current_goal: self.state.current_goal.clone(),
            decision_history: self.decision_history.clone(),
            state_info: format!(
                "Stuck: {}, Known resources: {}",
                self.state.stuck_counter,
                self.state.known_resources.len()
            ),
            last_decision: self.last_decision.as_ref().map(|d| d.reasoning.clone()),
        }
    }

    fn reset(&mut self) {
        self.state = AIState {
            current_goal: None,
            stuck_counter: 0,
            last_position: None,
            known_resources: HashMap::new(),
            recent_threats: Vec::new(),
        };
        self.last_decision = None;
        self.decision_history.clear();
    }
}

impl SimpleAI {
    pub fn new(id: Uuid, personality: Personality, difficulty: f32) -> Self {
        Self {
            id,
            personality,
            difficulty,
            hat_manager: HatManager::new(),
            last_decision: None,
            state: AIState {
                current_goal: None,
                stuck_counter: 0,
                last_position: None,
                known_resources: HashMap::new(),
                recent_threats: Vec::new(),
            },
        }
    }
}

impl AIController for SimpleAI {
    fn id(&self) -> Uuid {
        self.id
    }

    fn perceive(&self, game_view: &GameView) -> Perception {
        Perception::from_game_view(game_view, self.id)
    }

    fn decide(
        &mut self,
        perception: &Perception,
        messages: &[AIMessage],
        delta_time: f32,
    ) -> Decision {
        // Simple AI just picks random tasks
        self.hat_manager.update_hat(perception);
        let tasks = self.hat_manager.get_current_tasks(perception);

        let selected_task = if !tasks.is_empty() {
            let index = (perception.my_id.as_u128() as usize
                + perception.team_state.mech_health.len())
                % tasks.len();
            Some(tasks[index].clone())
        } else {
            None
        };

        Decision {
            chosen_action: selected_task.map(|t| t.action),
            confidence: self.difficulty * 0.5,
            reasoning: format!(
                "Simple AI - Hat: {}",
                self.hat_manager.get_active_hat().name()
            ),
            messages: Vec::new(),
        }
    }

    fn get_debug_info(&self) -> AIDebugInfo {
        AIDebugInfo {
            ai_id: self.id,
            current_hat: self.hat_manager.get_active_hat().name().to_string(),
            personality: format!("{:?}", self.personality),
            current_goal: self.state.current_goal.clone(),
            decision_history: Vec::new(),
            state_info: "Simple AI".to_string(),
            last_decision: self.last_decision.as_ref().map(|d| d.reasoning.clone()),
        }
    }

    fn reset(&mut self) {
        self.state = AIState {
            current_goal: None,
            stuck_counter: 0,
            last_position: None,
            known_resources: HashMap::new(),
            recent_threats: Vec::new(),
        };
        self.last_decision = None;
    }
}
