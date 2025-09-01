use crate::TaskAction;

/// AI personality types that affect decision making
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Personality {
    Aggressive,
    Defensive,
    Support,
    Balanced,
}

impl Personality {
    /// Get name suffix for AI player names
    pub fn name_suffix(&self) -> &'static str {
        match self {
            Personality::Aggressive => "Hunter",
            Personality::Defensive => "Guardian",
            Personality::Support => "Helper",
            Personality::Balanced => "Pilot",
        }
    }

    /// Get task preference multiplier based on personality
    pub fn task_preference(&self, action: &TaskAction) -> f32 {
        match (self, action) {
            // Aggressive personality preferences
            (Personality::Aggressive, TaskAction::AttackTarget { .. }) => 1.5,
            (Personality::Aggressive, TaskAction::OperateStation { station_type }) => {
                match station_type {
                    shared::StationType::WeaponLaser | shared::StationType::WeaponProjectile => 1.3,
                    _ => 0.8,
                }
            }
            (Personality::Aggressive, TaskAction::DefendPosition { .. }) => 0.6,
            (Personality::Aggressive, TaskAction::CollectResource { .. }) => 0.7,

            // Defensive personality preferences
            (Personality::Defensive, TaskAction::DefendPosition { .. }) => 1.5,
            (Personality::Defensive, TaskAction::OperateStation { station_type }) => {
                match station_type {
                    shared::StationType::Shield => 1.4,
                    shared::StationType::Repair => 1.3,
                    _ => 0.9,
                }
            }
            (Personality::Defensive, TaskAction::AttackTarget { .. }) => 0.6,
            (Personality::Defensive, TaskAction::MoveToPosition { .. }) => 0.8,

            // Support personality preferences
            (Personality::Support, TaskAction::CollectResource { .. }) => 1.4,
            (Personality::Support, TaskAction::OperateStation { station_type }) => {
                match station_type {
                    shared::StationType::Repair => 1.5,
                    shared::StationType::Upgrade => 1.3,
                    shared::StationType::Electrical => 1.2,
                    _ => 0.9,
                }
            }
            (Personality::Support, TaskAction::FollowPlayer { .. }) => 1.2,
            (Personality::Support, TaskAction::AttackTarget { .. }) => 0.5,

            // Balanced personality - no strong preferences
            (Personality::Balanced, _) => 1.0,

            // Default for unspecified combinations
            _ => 1.0,
        }
    }

    /// Get combat aggressiveness (0.0 to 1.0)
    pub fn combat_aggressiveness(&self) -> f32 {
        match self {
            Personality::Aggressive => 0.9,
            Personality::Defensive => 0.3,
            Personality::Support => 0.2,
            Personality::Balanced => 0.5,
        }
    }

    /// Get resource collection priority (0.0 to 1.0)
    pub fn resource_priority(&self) -> f32 {
        match self {
            Personality::Aggressive => 0.3,
            Personality::Defensive => 0.5,
            Personality::Support => 0.8,
            Personality::Balanced => 0.6,
        }
    }

    /// Get teamwork tendency (0.0 to 1.0)
    pub fn teamwork_tendency(&self) -> f32 {
        match self {
            Personality::Aggressive => 0.4,
            Personality::Defensive => 0.7,
            Personality::Support => 0.9,
            Personality::Balanced => 0.6,
        }
    }

    /// Get risk tolerance (0.0 to 1.0)
    pub fn risk_tolerance(&self) -> f32 {
        match self {
            Personality::Aggressive => 0.8,
            Personality::Defensive => 0.2,
            Personality::Support => 0.3,
            Personality::Balanced => 0.5,
        }
    }

    /// Get preferred combat range
    pub fn preferred_combat_range(&self) -> CombatRange {
        match self {
            Personality::Aggressive => CombatRange::Close,
            Personality::Defensive => CombatRange::Long,
            Personality::Support => CombatRange::Safe,
            Personality::Balanced => CombatRange::Medium,
        }
    }

    /// Get reaction to threats
    pub fn threat_reaction(&self, threat_severity: f32) -> ThreatReaction {
        match self {
            Personality::Aggressive => {
                if threat_severity > 0.8 {
                    ThreatReaction::TacticalRetreat
                } else {
                    ThreatReaction::Engage
                }
            }
            Personality::Defensive => {
                if threat_severity > 0.4 {
                    ThreatReaction::Retreat
                } else {
                    ThreatReaction::Defend
                }
            }
            Personality::Support => {
                if threat_severity > 0.3 {
                    ThreatReaction::Retreat
                } else {
                    ThreatReaction::Evade
                }
            }
            Personality::Balanced => {
                if threat_severity > 0.6 {
                    ThreatReaction::TacticalRetreat
                } else if threat_severity > 0.3 {
                    ThreatReaction::Defend
                } else {
                    ThreatReaction::Engage
                }
            }
        }
    }
}

/// Combat range preferences
#[derive(Debug, Clone, Copy)]
pub enum CombatRange {
    Close,  // < 20 tiles
    Medium, // 20-40 tiles
    Long,   // 40-60 tiles
    Safe,   // > 60 tiles or inside mech
}

/// Reactions to threats
#[derive(Debug, Clone, Copy)]
pub enum ThreatReaction {
    Engage,          // Attack the threat
    Defend,          // Hold position and defend
    Evade,           // Move perpendicular to avoid
    Retreat,         // Move away from threat
    TacticalRetreat, // Move to better position
}

/// Personality traits that can be mixed
#[derive(Debug, Clone)]
pub struct PersonalityTraits {
    pub aggression: f32,
    pub caution: f32,
    pub cooperation: f32,
    pub adaptability: f32,
    pub efficiency: f32,
}

impl PersonalityTraits {
    /// Create traits from a base personality
    pub fn from_personality(personality: Personality) -> Self {
        match personality {
            Personality::Aggressive => Self {
                aggression: 0.9,
                caution: 0.2,
                cooperation: 0.4,
                adaptability: 0.6,
                efficiency: 0.7,
            },
            Personality::Defensive => Self {
                aggression: 0.2,
                caution: 0.9,
                cooperation: 0.7,
                adaptability: 0.5,
                efficiency: 0.6,
            },
            Personality::Support => Self {
                aggression: 0.1,
                caution: 0.6,
                cooperation: 0.9,
                adaptability: 0.7,
                efficiency: 0.8,
            },
            Personality::Balanced => Self {
                aggression: 0.5,
                caution: 0.5,
                cooperation: 0.6,
                adaptability: 0.8,
                efficiency: 0.7,
            },
        }
    }

    /// Create a custom personality mix
    pub fn custom(
        aggression: f32,
        caution: f32,
        cooperation: f32,
        adaptability: f32,
        efficiency: f32,
    ) -> Self {
        Self {
            aggression: aggression.clamp(0.0, 1.0),
            caution: caution.clamp(0.0, 1.0),
            cooperation: cooperation.clamp(0.0, 1.0),
            adaptability: adaptability.clamp(0.0, 1.0),
            efficiency: efficiency.clamp(0.0, 1.0),
        }
    }

    /// Blend two personalities
    pub fn blend(a: &Self, b: &Self, weight: f32) -> Self {
        let w = weight.clamp(0.0, 1.0);
        let inv_w = 1.0 - w;

        Self {
            aggression: a.aggression * inv_w + b.aggression * w,
            caution: a.caution * inv_w + b.caution * w,
            cooperation: a.cooperation * inv_w + b.cooperation * w,
            adaptability: a.adaptability * inv_w + b.adaptability * w,
            efficiency: a.efficiency * inv_w + b.efficiency * w,
        }
    }

    /// Get decision weight based on traits
    pub fn decision_weight(&self, decision_type: DecisionType) -> f32 {
        match decision_type {
            DecisionType::Attack => self.aggression * (1.0 - self.caution * 0.5),
            DecisionType::Defend => self.caution * 0.8 + self.cooperation * 0.2,
            DecisionType::Support => self.cooperation * 0.9 + self.efficiency * 0.1,
            DecisionType::Explore => self.adaptability * 0.7 + (1.0 - self.caution) * 0.3,
            DecisionType::Optimize => self.efficiency * 0.8 + self.adaptability * 0.2,
        }
    }
}

/// Types of decisions for trait weighting
#[derive(Debug, Clone, Copy)]
pub enum DecisionType {
    Attack,
    Defend,
    Support,
    Explore,
    Optimize,
}
