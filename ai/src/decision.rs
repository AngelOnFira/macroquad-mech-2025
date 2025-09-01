use crate::{AICommand, AIMessage, TaskAction};
use shared::*;
use uuid::Uuid;

/// AI decision output
#[derive(Debug, Clone)]
pub struct Decision {
    pub chosen_action: Option<TaskAction>,
    pub confidence: f32,
    pub reasoning: String,
    pub messages: Vec<AIMessage>,
}

impl Decision {
    /// Convert decision to executable commands
    pub fn to_commands(&self, ai_id: Uuid) -> Vec<AICommand> {
        let mut commands = Vec::new();

        if let Some(action) = &self.chosen_action {
            match action {
                TaskAction::MoveToPosition { target, .. } => {
                    // Calculate movement direction
                    // This is simplified - in reality would need current position
                    let movement = (
                        target.x.signum() * self.confidence,
                        target.y.signum() * self.confidence,
                    );

                    commands.push(AICommand::Move {
                        player_id: ai_id,
                        movement,
                    });
                }

                TaskAction::OperateStation { station_type } => {
                    // Map station type to button index
                    let button_index = match station_type {
                        StationType::WeaponLaser => 0,
                        StationType::WeaponProjectile => 0,
                        StationType::Shield => 0,
                        StationType::Repair => 0,
                        StationType::Upgrade => 0, // Would need more logic for upgrade buttons
                        _ => 0,
                    };

                    commands.push(AICommand::PressButton {
                        player_id: ai_id,
                        button_index,
                    });
                }

                TaskAction::CollectResource { .. } => {
                    // Move towards resource (simplified)
                    commands.push(AICommand::Move {
                        player_id: ai_id,
                        movement: (0.5, 0.5), // Would calculate actual direction
                    });
                }

                TaskAction::FollowPlayer { .. } => {
                    // Move towards player (simplified)
                    commands.push(AICommand::Move {
                        player_id: ai_id,
                        movement: (0.3, 0.3), // Would calculate actual direction
                    });
                }

                TaskAction::AttackTarget { .. } => {
                    // Press fire button if at a weapon station
                    commands.push(AICommand::PressButton {
                        player_id: ai_id,
                        button_index: 0,
                    });
                }

                TaskAction::DefendPosition { .. } => {
                    // Stay in position, maybe operate defensive stations
                    // No movement command
                }

                TaskAction::RepairMech => {
                    // Press repair button
                    commands.push(AICommand::PressButton {
                        player_id: ai_id,
                        button_index: 0,
                    });
                }

                TaskAction::Idle => {
                    // Do nothing
                }
            }
        }

        commands
    }
}

/// Debug information for AI state
#[derive(Debug, Clone)]
pub struct AIDebugInfo {
    pub ai_id: Uuid,
    pub current_hat: String,
    pub personality: String,
    pub current_goal: Option<String>,
    pub decision_history: Vec<(String, f32)>,
    pub state_info: String,
    pub last_decision: Option<String>,
}

/// Decision context for more complex decisions
#[derive(Debug, Clone)]
pub struct DecisionContext {
    pub time_since_last_decision: f32,
    pub repeated_action_count: u32,
    pub team_coordination_score: f32,
    pub stress_level: f32,
}

impl DecisionContext {
    pub fn new() -> Self {
        Self {
            time_since_last_decision: 0.0,
            repeated_action_count: 0,
            team_coordination_score: 1.0,
            stress_level: 0.0,
        }
    }

    /// Update context based on new decision
    pub fn update(&mut self, decision: &Decision, delta_time: f32) {
        self.time_since_last_decision += delta_time;

        // Track repeated actions
        // In a real implementation, would compare with previous decisions

        // Update stress based on confidence
        self.stress_level = (1.0 - decision.confidence) * 0.5 + self.stress_level * 0.5;
    }
}

/// Decision modifiers based on difficulty
pub struct DifficultyModifiers {
    pub reaction_time: f32,   // Delay before making decisions
    pub accuracy: f32,        // How optimal decisions are
    pub coordination: f32,    // How well AI works with team
    pub adaptation_rate: f32, // How quickly AI learns
    pub mistake_chance: f32,  // Chance of making mistakes
}

impl DifficultyModifiers {
    pub fn from_difficulty(difficulty: f32) -> Self {
        let d = difficulty.clamp(0.0, 1.0);

        Self {
            reaction_time: 1.0 - d * 0.8,    // 1.0s to 0.2s
            accuracy: 0.5 + d * 0.5,         // 50% to 100%
            coordination: 0.3 + d * 0.7,     // 30% to 100%
            adaptation_rate: 0.2 + d * 0.8,  // 20% to 100%
            mistake_chance: 0.3 * (1.0 - d), // 30% to 0%
        }
    }

    /// Apply modifiers to a decision
    pub fn apply(&self, decision: &mut Decision) {
        // Reduce confidence based on accuracy
        decision.confidence *= self.accuracy;

        // Sometimes make mistakes
        if rand::random::<f32>() < self.mistake_chance {
            decision.confidence *= 0.5;
            decision.reasoning.push_str(" [Mistake]");
        }
    }
}
