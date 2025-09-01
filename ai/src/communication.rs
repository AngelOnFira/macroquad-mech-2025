use chrono::{DateTime, Utc};
use shared::*;
use std::collections::VecDeque;
use uuid::Uuid;

/// Message that AIs can send to each other
#[derive(Debug, Clone)]
pub struct AIMessage {
    pub id: Uuid,
    pub sender: Uuid,
    pub recipient: Option<Uuid>, // None = broadcast to team
    pub message_type: MessageType,
    pub priority: MessagePriority,
    pub timestamp: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Types of messages AIs can send
#[derive(Debug, Clone)]
pub enum MessageType {
    // Commands (from captain)
    Command { order: Order },

    // Status updates
    StatusUpdate { status: Status },

    // Requests
    Request { request_type: RequestType },

    // Information sharing
    Intel { info: IntelInfo },

    // Coordination
    Coordination { action: CoordinationAction },
}

/// Orders that can be given (usually by captain)
#[derive(Debug, Clone)]
pub enum Order {
    MoveTo {
        position: WorldPos,
        urgency: Urgency,
    },
    OperateStation {
        station_type: StationType,
    },
    CollectResources {
        resource_type: Option<ResourceType>,
    },
    DefendPosition {
        position: WorldPos,
    },
    AttackTarget {
        target_id: Uuid,
    },
    Retreat,
    FormUp,
}

/// Status updates
#[derive(Debug, Clone)]
pub enum Status {
    ChangingHat {
        new_hat: String,
    },
    UnderAttack {
        threat_level: f32,
    },
    LowHealth {
        health_percent: f32,
    },
    ResourceFound {
        position: WorldPos,
        resource_type: ResourceType,
    },
    StationAvailable {
        station_type: StationType,
        mech_id: Uuid,
    },
    TaskCompleted {
        task: String,
    },
}

/// Types of requests
#[derive(Debug, Clone)]
pub enum RequestType {
    NeedHelp {
        position: WorldPos,
        urgency: Urgency,
    },
    NeedResources {
        resource_type: ResourceType,
        amount: u32,
    },
    NeedBackup {
        enemy_count: usize,
    },
    RequestRole {
        preferred_hat: String,
    },
}

/// Intelligence information
#[derive(Debug, Clone)]
pub enum IntelInfo {
    EnemySpotted {
        position: WorldPos,
        enemy_type: String,
    },
    ResourceLocation {
        position: WorldPos,
        resource_type: ResourceType,
    },
    MechStatus {
        mech_id: Uuid,
        health: u32,
        shield: u32,
    },
    StrategicPosition {
        position: WorldPos,
        value: f32,
    },
}

/// Coordination actions
#[derive(Debug, Clone)]
pub enum CoordinationAction {
    MovingTo { position: WorldPos, eta: f32 },
    LeavingStation { station_type: StationType },
    ClaimingResource { resource_id: Uuid },
    EngagingEnemy { target_id: Uuid },
}

/// Message priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Urgency levels
#[derive(Debug, Clone, Copy)]
pub enum Urgency {
    Low,
    Normal,
    High,
    Critical,
}

/// Communication system that manages AI messages
pub struct CommunicationSystem {
    messages: VecDeque<AIMessage>,
    captain: Option<Uuid>,
    message_history: Vec<AIMessage>,
    max_history: usize,
}

impl CommunicationSystem {
    pub fn new(enable_captain: bool) -> Self {
        Self {
            messages: VecDeque::new(),
            captain: None,
            message_history: Vec::new(),
            max_history: 1000,
        }
    }

    /// Assign a captain
    pub fn assign_captain(&mut self, ai_id: Uuid) {
        self.captain = Some(ai_id);

        // Announce new captain
        self.send_message(
            ai_id,
            AIMessage {
                id: Uuid::new_v4(),
                sender: ai_id,
                recipient: None,
                message_type: MessageType::StatusUpdate {
                    status: Status::ChangingHat {
                        new_hat: "Captain".to_string(),
                    },
                },
                priority: MessagePriority::High,
                timestamp: Utc::now(),
                expires_at: None,
            },
        );
    }

    /// Send a message
    pub fn send_message(&mut self, sender: Uuid, mut message: AIMessage) {
        message.sender = sender;
        message.timestamp = Utc::now();

        // Captain messages get higher priority
        if Some(sender) == self.captain && message.priority < MessagePriority::High {
            message.priority = MessagePriority::High;
        }

        self.messages.push_back(message.clone());
        self.message_history.push(message);

        // Trim history if needed
        if self.message_history.len() > self.max_history {
            self.message_history.remove(0);
        }
    }

    /// Get pending messages and clear the queue
    pub fn get_pending_messages(&mut self) -> Vec<AIMessage> {
        let now = Utc::now();

        // Filter out expired messages
        self.messages
            .retain(|msg| msg.expires_at.map(|exp| exp > now).unwrap_or(true));

        // Sort by priority (highest first)
        let mut messages: Vec<_> = self.messages.drain(..).collect();
        messages.sort_by(|a, b| b.priority.cmp(&a.priority));

        messages
    }

    /// Check if an AI is the captain
    pub fn is_captain(&self, ai_id: Uuid) -> bool {
        self.captain == Some(ai_id)
    }

    /// Get message history for debugging
    pub fn get_history(&self) -> &[AIMessage] {
        &self.message_history
    }

    /// Create a standard message
    pub fn create_message(
        sender: Uuid,
        message_type: MessageType,
        priority: MessagePriority,
        recipient: Option<Uuid>,
    ) -> AIMessage {
        AIMessage {
            id: Uuid::new_v4(),
            sender,
            recipient,
            message_type,
            priority,
            timestamp: Utc::now(),
            expires_at: None,
        }
    }
}

/// Extension methods for AI to handle messages
pub trait MessageHandler {
    /// Process a message and decide how to respond
    fn handle_message(&mut self, message: &AIMessage, is_captain: bool) -> Option<MessageResponse>;

    /// Generate messages based on current state
    fn generate_messages(&self) -> Vec<AIMessage>;
}

/// Response to a message
#[derive(Debug)]
pub enum MessageResponse {
    /// Acknowledge and follow order
    Accept,
    /// Acknowledge but can't comply
    Decline { reason: String },
    /// Need more information
    RequestClarification,
    /// Counter-proposal
    Negotiate { alternative: String },
}

/// Helper functions for message creation
impl AIMessage {
    /// Create a command message
    pub fn command(sender: Uuid, order: Order, recipient: Option<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender,
            recipient,
            message_type: MessageType::Command { order },
            priority: MessagePriority::High,
            timestamp: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::seconds(30)),
        }
    }

    /// Create a status update
    pub fn status(sender: Uuid, status: Status) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender,
            recipient: None,
            message_type: MessageType::StatusUpdate { status },
            priority: MessagePriority::Normal,
            timestamp: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::seconds(10)),
        }
    }

    /// Create a request
    pub fn request(sender: Uuid, request_type: RequestType) -> Self {
        let priority = match &request_type {
            RequestType::NeedHelp { urgency, .. } => match urgency {
                Urgency::Critical => MessagePriority::Critical,
                Urgency::High => MessagePriority::High,
                _ => MessagePriority::Normal,
            },
            RequestType::NeedBackup { .. } => MessagePriority::High,
            _ => MessagePriority::Normal,
        };

        Self {
            id: Uuid::new_v4(),
            sender,
            recipient: None,
            message_type: MessageType::Request { request_type },
            priority,
            timestamp: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::seconds(20)),
        }
    }

    /// Create an intel message
    pub fn intel(sender: Uuid, info: IntelInfo) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender,
            recipient: None,
            message_type: MessageType::Intel { info },
            priority: MessagePriority::Normal,
            timestamp: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::seconds(60)),
        }
    }

    /// Create a coordination message
    pub fn coordinate(sender: Uuid, action: CoordinationAction) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender,
            recipient: None,
            message_type: MessageType::Coordination { action },
            priority: MessagePriority::Normal,
            timestamp: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::seconds(15)),
        }
    }
}
