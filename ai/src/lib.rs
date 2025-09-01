pub mod communication;
pub mod decision;
pub mod hats;
pub mod interface;
pub mod logging;
pub mod perception;
pub mod personality;
pub mod utility;

use shared::*;
use std::collections::HashMap;
use uuid::Uuid;

pub use communication::*;
pub use decision::*;
pub use hats::*;
pub use interface::*;
pub use logging::*;
pub use perception::*;
pub use personality::*;
pub use utility::*;

/// Configuration for AI system
#[derive(Debug, Clone)]
pub struct AIConfig {
    /// How many AIs to spawn
    pub ai_count: usize,
    /// Difficulty level (0.0 = easy, 1.0 = hard)
    pub difficulty: f32,
    /// Whether to enable captain role
    pub enable_captain: bool,
    /// Update frequency (times per second)
    pub update_frequency: f32,
    /// Enable debug logging
    pub debug_logging: bool,
    /// Team assignment
    pub team: TeamId,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            ai_count: 3,
            difficulty: 0.5,
            enable_captain: false,
            update_frequency: 10.0,
            debug_logging: true,
            team: TeamId::Red,
        }
    }
}

/// Main AI manager that coordinates all AI players
pub struct AIManager {
    /// All AI controllers
    controllers: HashMap<Uuid, Box<dyn AIController>>,
    /// Communication system
    comm_system: CommunicationSystem,
    /// Decision logger
    logger: DecisionLogger,
    /// Configuration
    config: AIConfig,
}

impl AIManager {
    /// Create a new AI manager
    pub fn new(config: AIConfig) -> Self {
        Self {
            controllers: HashMap::new(),
            comm_system: CommunicationSystem::new(config.enable_captain),
            logger: DecisionLogger::new(config.debug_logging),
            config,
        }
    }

    /// Initialize AI players
    pub fn initialize_ais(&mut self) -> Vec<(String, TeamId)> {
        let mut ai_players = Vec::new();

        for i in 0..self.config.ai_count {
            let ai_id = Uuid::new_v4();
            let personality = self.select_personality(i);
            let name = format!("AI_{}", personality.name_suffix());

            // Create controller based on difficulty
            let controller: Box<dyn AIController> = if self.config.difficulty > 0.7 {
                Box::new(utility::UtilityAI::new(
                    ai_id,
                    personality,
                    self.config.difficulty,
                ))
            } else {
                Box::new(utility::SimpleAI::new(
                    ai_id,
                    personality,
                    self.config.difficulty,
                ))
            };

            self.controllers.insert(ai_id, controller);
            ai_players.push((name, self.config.team));
        }

        // Assign captain if enabled
        if self.config.enable_captain && !self.controllers.is_empty() {
            let captain_id = self.controllers.keys().next().cloned().unwrap();
            self.comm_system.assign_captain(captain_id);
        }

        ai_players
    }

    /// Add a single AI with specific personality and difficulty
    pub fn add_ai(&mut self, personality: Personality, difficulty: f32) -> Uuid {
        let ai_id = Uuid::new_v4();

        // Create controller based on difficulty
        let controller: Box<dyn AIController> = if difficulty > 0.7 {
            Box::new(utility::UtilityAI::new(ai_id, personality, difficulty))
        } else {
            Box::new(utility::SimpleAI::new(ai_id, personality, difficulty))
        };

        self.controllers.insert(ai_id, controller);
        ai_id
    }

    /// Remove an AI by ID
    pub fn remove_ai(&mut self, ai_id: Uuid) {
        self.controllers.remove(&ai_id);
    }

    /// Update all AIs
    pub fn update(&mut self, game_view: &GameView, delta_time: f32) -> Vec<AICommand> {
        let mut all_commands = Vec::new();

        // Collect perceptions for all AIs
        let mut perceptions = HashMap::new();
        for (ai_id, controller) in &self.controllers {
            let perception = controller.perceive(game_view);
            perceptions.insert(*ai_id, perception);
        }

        // Process communications
        let messages = self.comm_system.get_pending_messages();

        // Update each AI
        for (ai_id, controller) in &mut self.controllers {
            if let Some(perception) = perceptions.get(ai_id) {
                // Let AI process messages
                let relevant_messages: Vec<_> = messages
                    .iter()
                    .filter(|m| m.recipient.is_none() || m.recipient == Some(*ai_id))
                    .cloned()
                    .collect();

                // Get AI decision
                let decision = controller.decide(perception, &relevant_messages, delta_time);

                // Log decision
                self.logger.log_decision(*ai_id, &decision);

                // Convert decision to commands
                let commands = decision.to_commands(*ai_id);
                all_commands.extend(commands);

                // Handle any communications the AI wants to send
                for msg in decision.messages {
                    self.comm_system.send_message(*ai_id, msg);
                }
            }
        }

        all_commands
    }

    /// Get debug info for a specific AI
    pub fn get_debug_info(&self, ai_id: Uuid) -> Option<AIDebugInfo> {
        self.controllers
            .get(&ai_id)
            .map(|controller| controller.get_debug_info())
    }

    /// Select personality based on index
    fn select_personality(&self, index: usize) -> Personality {
        match index % 4 {
            0 => Personality::Aggressive,
            1 => Personality::Defensive,
            2 => Personality::Support,
            _ => Personality::Balanced,
        }
    }
}

/// Commands that AI can issue
#[derive(Debug, Clone)]
pub enum AICommand {
    Move {
        player_id: Uuid,
        movement: (f32, f32),
    },
    PressButton {
        player_id: Uuid,
        button_index: u8,
    },
    ExitMech {
        player_id: Uuid,
    },
    EngineControl {
        player_id: Uuid,
        movement: (f32, f32),
    },
}
