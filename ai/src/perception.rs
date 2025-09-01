use crate::{GameView, MechView, PlayerView, ProjectileView, TeamInfo};
use shared::*;
use std::collections::HashMap;
use uuid::Uuid;

/// AI's perception of the game state
#[derive(Debug, Clone)]
pub struct Perception {
    /// My player ID
    pub my_id: Uuid,
    /// My current state
    pub my_state: MyState,
    /// Nearby threats
    pub threats: Vec<Threat>,
    /// Available opportunities
    pub opportunities: Vec<Opportunity>,
    /// Team state
    pub team_state: TeamState,
    /// Environmental awareness
    pub environment: EnvironmentInfo,
}

/// AI's own state
#[derive(Debug, Clone)]
pub struct MyState {
    pub location: PlayerLocation,
    pub carrying_resource: Option<ResourceType>,
    pub operating_station: Option<StationType>,
    pub health_status: HealthStatus,
    pub nearest_safe_location: Option<WorldPos>,
}

/// Health status
#[derive(Debug, Clone, Copy)]
pub enum HealthStatus {
    Healthy,
    Damaged,
    Critical,
    Dead,
}

/// Threat information
#[derive(Debug, Clone)]
pub struct Threat {
    pub threat_type: ThreatType,
    pub position: WorldPos,
    pub severity: f32, // 0.0 to 1.0
    pub distance: f32,
    pub time_to_impact: Option<f32>,
}

#[derive(Debug, Clone)]
pub enum ThreatType {
    EnemyMech { id: Uuid, health: u32 },
    Projectile { id: Uuid, velocity: (f32, f32) },
    EnvironmentalHazard,
}

/// Opportunity information
#[derive(Debug, Clone)]
pub struct Opportunity {
    pub opportunity_type: OpportunityType,
    pub position: WorldPos,
    pub value: f32, // 0.0 to 1.0
    pub distance: f32,
    pub time_estimate: f32,
}

#[derive(Debug, Clone)]
pub enum OpportunityType {
    Resource { resource_type: ResourceType },
    UnmannedStation { station_type: StationType },
    WeakEnemy { id: Uuid, health: u32 },
    TeamObjective { description: String },
}

/// Team state information
#[derive(Debug, Clone)]
pub struct TeamState {
    pub mech_health: HashMap<Uuid, (u32, u32)>, // (health, shield)
    pub player_roles: HashMap<Uuid, String>,    // Current "hat" each player is wearing
    pub resource_status: ResourceStatus,
    pub combat_readiness: f32, // 0.0 to 1.0
}

#[derive(Debug, Clone)]
pub struct ResourceStatus {
    pub total_resources: HashMap<ResourceType, u32>,
    pub resource_needs: HashMap<ResourceType, u32>,
    pub scarcity_level: f32, // 0.0 to 1.0
}

/// Environmental information
#[derive(Debug, Clone)]
pub struct EnvironmentInfo {
    pub nearby_resources: Vec<(WorldPos, ResourceType)>,
    pub safe_zones: Vec<WorldPos>,
    pub contested_areas: Vec<WorldPos>,
    pub strategic_positions: Vec<WorldPos>,
}

impl Perception {
    /// Create perception from game view
    pub fn from_game_view(game_view: &GameView, ai_id: Uuid) -> Self {
        let my_player = game_view.players.iter().find(|p| p.id == ai_id).cloned();

        let my_state = if let Some(player) = my_player {
            MyState {
                location: player.location,
                carrying_resource: player.carrying_resource,
                operating_station: player.operating_station,
                health_status: HealthStatus::Healthy, // TODO: Track actual health
                nearest_safe_location: find_nearest_safe_location(&game_view, &player),
            }
        } else {
            MyState {
                location: PlayerLocation::OutsideWorld(WorldPos::new(0.0, 0.0)),
                carrying_resource: None,
                operating_station: None,
                health_status: HealthStatus::Dead,
                nearest_safe_location: None,
            }
        };

        let threats = identify_threats(&game_view, ai_id, &my_state);
        let opportunities = identify_opportunities(&game_view, ai_id, &my_state);
        let team_state = analyze_team_state(&game_view, ai_id);
        let environment = analyze_environment(&game_view);

        Perception {
            my_id: ai_id,
            my_state,
            threats,
            opportunities,
            team_state,
            environment,
        }
    }
}

/// Find nearest safe location
fn find_nearest_safe_location(game_view: &GameView, player: &PlayerView) -> Option<WorldPos> {
    // Find friendly mech
    let friendly_mechs: Vec<_> = game_view
        .mechs
        .iter()
        .filter(|m| m.team == player.team)
        .collect();

    if let PlayerLocation::OutsideWorld(pos) = player.location {
        friendly_mechs.iter().map(|m| m.position).min_by(|a, b| {
            let dist_a = pos.distance_to(*a);
            let dist_b = pos.distance_to(*b);
            dist_a.partial_cmp(&dist_b).unwrap()
        })
    } else {
        None
    }
}

/// Identify threats
fn identify_threats(game_view: &GameView, ai_id: Uuid, my_state: &MyState) -> Vec<Threat> {
    let mut threats = Vec::new();

    let my_pos = match my_state.location {
        PlayerLocation::OutsideWorld(pos) => Some(pos),
        PlayerLocation::InsideMech { mech_id, .. } => game_view
            .mechs
            .iter()
            .find(|m| m.id == mech_id)
            .map(|m| m.position),
    };

    if let Some(pos) = my_pos {
        // Enemy mechs
        for mech in &game_view.mechs {
            if mech.team != game_view.team_info.team_id {
                let distance = pos.distance_to(mech.position);
                let severity = calculate_mech_threat_severity(mech, distance);

                threats.push(Threat {
                    threat_type: ThreatType::EnemyMech {
                        id: mech.id,
                        health: mech.health,
                    },
                    position: mech.position,
                    severity,
                    distance,
                    time_to_impact: None,
                });
            }
        }

        // Enemy projectiles
        for projectile in &game_view.projectiles {
            if projectile.owner_team != game_view.team_info.team_id {
                let distance = pos.distance_to(projectile.position);
                let (severity, time_to_impact) = calculate_projectile_threat(projectile, pos);

                if severity > 0.0 {
                    threats.push(Threat {
                        threat_type: ThreatType::Projectile {
                            id: projectile.id,
                            velocity: projectile.velocity,
                        },
                        position: projectile.position,
                        severity,
                        distance,
                        time_to_impact: Some(time_to_impact),
                    });
                }
            }
        }
    }

    // Sort by severity
    threats.sort_by(|a, b| b.severity.partial_cmp(&a.severity).unwrap());
    threats
}

/// Calculate mech threat severity
fn calculate_mech_threat_severity(mech: &MechView, distance: f32) -> f32 {
    let health_factor = mech.health as f32 / 100.0;
    let shield_factor = mech.shield as f32 / 100.0;
    let distance_factor = (100.0 - distance.min(100.0)) / 100.0;

    (health_factor + shield_factor) * 0.5 * distance_factor
}

/// Calculate projectile threat
fn calculate_projectile_threat(projectile: &ProjectileView, my_pos: WorldPos) -> (f32, f32) {
    // Simple calculation - check if projectile is heading towards us
    let to_me = (
        my_pos.x - projectile.position.x,
        my_pos.y - projectile.position.y,
    );
    let distance = (to_me.0 * to_me.0 + to_me.1 * to_me.1).sqrt();

    if distance < 0.1 {
        return (1.0, 0.0);
    }

    // Dot product to see if projectile is heading towards us
    let dot = to_me.0 * projectile.velocity.0 + to_me.1 * projectile.velocity.1;
    let projectile_speed = (projectile.velocity.0.powi(2) + projectile.velocity.1.powi(2)).sqrt();

    if projectile_speed > 0.0 && dot > 0.0 {
        let time_to_impact = distance / projectile_speed;
        let severity =
            (1.0 - (time_to_impact / 5.0).min(1.0)) * (dot / (distance * projectile_speed));
        (severity.max(0.0), time_to_impact)
    } else {
        (0.0, f32::MAX)
    }
}

/// Identify opportunities
fn identify_opportunities(
    game_view: &GameView,
    ai_id: Uuid,
    my_state: &MyState,
) -> Vec<Opportunity> {
    let mut opportunities = Vec::new();

    let my_pos = match my_state.location {
        PlayerLocation::OutsideWorld(pos) => Some(pos),
        PlayerLocation::InsideMech { mech_id, .. } => game_view
            .mechs
            .iter()
            .find(|m| m.id == mech_id)
            .map(|m| m.position),
    };

    if let Some(pos) = my_pos {
        // Resources
        for resource in &game_view.resources {
            let distance = pos.distance_to(resource.position);
            let value = calculate_resource_value(resource.resource_type, &game_view.team_info);

            opportunities.push(Opportunity {
                opportunity_type: OpportunityType::Resource {
                    resource_type: resource.resource_type,
                },
                position: resource.position,
                value,
                distance,
                time_estimate: distance / (PLAYER_MOVE_SPEED * TILE_SIZE),
            });
        }

        // Unmanned stations
        for mech in game_view
            .mechs
            .iter()
            .filter(|m| m.team == game_view.team_info.team_id)
        {
            for station in &mech.stations {
                if station.operated_by.is_none() {
                    let station_world_pos = mech.position; // TODO: Calculate actual station position
                    let distance = pos.distance_to(station_world_pos);
                    let value = calculate_station_value(station.station_type);

                    opportunities.push(Opportunity {
                        opportunity_type: OpportunityType::UnmannedStation {
                            station_type: station.station_type,
                        },
                        position: station_world_pos,
                        value,
                        distance,
                        time_estimate: distance / (PLAYER_MOVE_SPEED * TILE_SIZE) + 2.0,
                    });
                }
            }
        }
    }

    // Sort by value/distance ratio
    opportunities.sort_by(|a, b| {
        let ratio_a = a.value / (a.distance + 1.0);
        let ratio_b = b.value / (b.distance + 1.0);
        ratio_b.partial_cmp(&ratio_a).unwrap()
    });

    opportunities
}

/// Calculate resource value based on team needs
fn calculate_resource_value(resource_type: ResourceType, team_info: &TeamInfo) -> f32 {
    let current_count = team_info.total_resources.get(&resource_type).unwrap_or(&0);

    // Higher value for resources we have less of
    match resource_type {
        ResourceType::ScrapMetal => 0.6 - (*current_count as f32 * 0.05).min(0.4),
        ResourceType::ComputerComponents => 0.8 - (*current_count as f32 * 0.1).min(0.6),
        ResourceType::Batteries => 0.9 - (*current_count as f32 * 0.1).min(0.7),
        ResourceType::Wiring => 0.7 - (*current_count as f32 * 0.08).min(0.5),
    }
}

/// Calculate station value
fn calculate_station_value(station_type: StationType) -> f32 {
    match station_type {
        StationType::Engine => 0.9,
        StationType::WeaponLaser | StationType::WeaponProjectile => 0.8,
        StationType::Shield => 0.7,
        StationType::Repair => 0.6,
        StationType::Upgrade => 0.5,
        StationType::Electrical => 0.4,
        StationType::Pilot => 0.85, // High value for strategic control
    }
}

/// Analyze team state
fn analyze_team_state(game_view: &GameView, ai_id: Uuid) -> TeamState {
    let mut mech_health = HashMap::new();
    let mut player_roles = HashMap::new();

    // Collect mech health
    for mech in game_view
        .mechs
        .iter()
        .filter(|m| m.team == game_view.team_info.team_id)
    {
        mech_health.insert(mech.id, (mech.health, mech.shield));
    }

    // Guess player roles based on their actions
    for player in game_view
        .players
        .iter()
        .filter(|p| p.team == game_view.team_info.team_id)
    {
        let role = if player.operating_station.is_some() {
            "Operator".to_string()
        } else if player.carrying_resource.is_some() {
            "Scavenger".to_string()
        } else {
            "Scout".to_string()
        };
        player_roles.insert(player.id, role);
    }

    // Calculate resource needs
    let mut resource_needs = HashMap::new();
    resource_needs.insert(ResourceType::ScrapMetal, 5);
    resource_needs.insert(ResourceType::ComputerComponents, 3);
    resource_needs.insert(ResourceType::Batteries, 3);
    resource_needs.insert(ResourceType::Wiring, 4);

    let total_resources = game_view.team_info.total_resources.values().sum::<u32>();
    let world_resources = game_view.resources.len() as u32;
    let available_resources = total_resources + world_resources;
    let scarcity_level = 1.0 - (available_resources as f32 / 20.0).min(1.0);

    let resource_status = ResourceStatus {
        total_resources: game_view.team_info.total_resources.clone(),
        resource_needs,
        scarcity_level,
    };

    // Calculate combat readiness
    let avg_mech_health = mech_health
        .values()
        .map(|(h, s)| (*h + *s) as f32 / 200.0)
        .sum::<f32>()
        / mech_health.len().max(1) as f32;

    let combat_readiness = avg_mech_health * (1.0 - scarcity_level * 0.5);

    TeamState {
        mech_health,
        player_roles,
        resource_status,
        combat_readiness,
    }
}

/// Analyze environment
fn analyze_environment(game_view: &GameView) -> EnvironmentInfo {
    let mut nearby_resources = Vec::new();
    let mut safe_zones = Vec::new();
    let mut contested_areas = Vec::new();
    let strategic_positions = vec![WorldPos::new(
        ARENA_WIDTH_TILES as f32 * TILE_SIZE / 2.0,
        ARENA_HEIGHT_TILES as f32 * TILE_SIZE / 2.0,
    )];

    // Collect resource positions
    for resource in &game_view.resources {
        nearby_resources.push((resource.position, resource.resource_type));
    }

    // Identify safe zones (near friendly mechs)
    for mech in game_view
        .mechs
        .iter()
        .filter(|m| m.team == game_view.team_info.team_id)
    {
        safe_zones.push(mech.position);
    }

    // Identify contested areas (between friendly and enemy mechs)
    for friendly in game_view
        .mechs
        .iter()
        .filter(|m| m.team == game_view.team_info.team_id)
    {
        for enemy in game_view
            .mechs
            .iter()
            .filter(|m| m.team != game_view.team_info.team_id)
        {
            let mid_point = WorldPos::new(
                (friendly.position.x + enemy.position.x) / 2.0,
                (friendly.position.y + enemy.position.y) / 2.0,
            );
            contested_areas.push(mid_point);
        }
    }

    EnvironmentInfo {
        nearby_resources,
        safe_zones,
        contested_areas,
        strategic_positions,
    }
}
