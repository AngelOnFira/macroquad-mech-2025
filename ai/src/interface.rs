use uuid::Uuid;
use shared::*;
use crate::{Perception, Decision, AIMessage, AIDebugInfo};

/// Abstract interface for AI controllers
pub trait AIController: Send + Sync {
    /// Get AI's unique ID
    fn id(&self) -> Uuid;
    
    /// Perceive the game state
    fn perceive(&self, game_view: &GameView) -> Perception;
    
    /// Make a decision based on perception and messages
    fn decide(&mut self, perception: &Perception, messages: &[AIMessage], delta_time: f32) -> Decision;
    
    /// Get debug information about the AI's current state
    fn get_debug_info(&self) -> AIDebugInfo;
    
    /// Reset AI state (useful for respawning)
    fn reset(&mut self);
}

/// View of the game state from AI's perspective
#[derive(Debug, Clone)]
pub struct GameView {
    /// Current tick
    pub tick: u64,
    /// All players
    pub players: Vec<PlayerView>,
    /// All mechs
    pub mechs: Vec<MechView>,
    /// Visible resources
    pub resources: Vec<ResourceView>,
    /// Visible projectiles
    pub projectiles: Vec<ProjectileView>,
    /// Team information
    pub team_info: TeamInfo,
}

/// Player information visible to AI
#[derive(Debug, Clone)]
pub struct PlayerView {
    pub id: Uuid,
    pub name: String,
    pub team: TeamId,
    pub location: PlayerLocation,
    pub carrying_resource: Option<ResourceType>,
    pub operating_station: Option<StationType>,
    pub is_self: bool,
}

/// Mech information visible to AI
#[derive(Debug, Clone)]
pub struct MechView {
    pub id: Uuid,
    pub team: TeamId,
    pub position: WorldPos,
    pub health: u32,
    pub shield: u32,
    pub velocity: (f32, f32),
    pub stations: Vec<StationView>,
    pub resource_inventory: std::collections::HashMap<ResourceType, u32>,
}

/// Station information visible to AI
#[derive(Debug, Clone)]
pub struct StationView {
    pub id: Uuid,
    pub station_type: StationType,
    pub operated_by: Option<Uuid>,
    pub position: TilePos,
    pub floor: u8,
}

/// Resource information visible to AI
#[derive(Debug, Clone)]
pub struct ResourceView {
    pub id: Uuid,
    pub position: WorldPos,
    pub resource_type: ResourceType,
}

/// Projectile information visible to AI
#[derive(Debug, Clone)]
pub struct ProjectileView {
    pub id: Uuid,
    pub position: WorldPos,
    pub velocity: (f32, f32),
    pub owner_team: TeamId,
}

/// Team information
#[derive(Debug, Clone)]
pub struct TeamInfo {
    pub team_id: TeamId,
    pub player_count: usize,
    pub mech_count: usize,
    pub total_resources: std::collections::HashMap<ResourceType, u32>,
}

/// Convert server game state to AI game view
pub fn create_game_view(
    game_state: &ServerMessage,
    ai_player_id: Uuid,
    ai_team: TeamId,
) -> Option<GameView> {
    if let ServerMessage::GameState { players, mechs, resources, projectiles } = game_state {
        let player_views: Vec<PlayerView> = players.values()
            .map(|p| {
                // Find the station type if player is operating a station
                let operating_station_type = if let Some(station_id) = p.operating_station {
                    mechs.values()
                        .flat_map(|m| &m.stations)
                        .find(|s| s.id == station_id)
                        .map(|s| s.station_type)
                } else {
                    None
                };
                
                PlayerView {
                    id: p.id,
                    name: p.name.clone(),
                    team: p.team,
                    location: p.location,
                    carrying_resource: p.carrying_resource,
                    operating_station: operating_station_type,
                    is_self: p.id == ai_player_id,
                }
            })
            .collect();
        
        let mech_views: Vec<MechView> = mechs.values()
            .map(|m| MechView {
                id: m.id,
                team: m.team,
                position: m.world_position,
                health: m.health,
                shield: m.shield,
                velocity: (0.0, 0.0), // TODO: Calculate from position changes
                stations: m.stations.iter()
                    .map(|s| StationView {
                        id: s.id,
                        station_type: s.station_type,
                        operated_by: s.operated_by,
                        position: s.position,
                        floor: s.floor,
                    })
                    .collect(),
                resource_inventory: m.resource_inventory.clone(),
            })
            .collect();
        
        let resource_views: Vec<ResourceView> = resources.iter()
            .map(|r| ResourceView {
                id: r.id,
                position: r.position.to_world_pos(),
                resource_type: r.resource_type,
            })
            .collect();
        
        let projectile_views: Vec<ProjectileView> = projectiles.iter()
            .map(|p| ProjectileView {
                id: p.id,
                position: p.position,
                velocity: p.velocity,
                owner_team: mechs.values()
                    .find(|m| m.id == p.owner_mech_id)
                    .map(|m| m.team)
                    .unwrap_or(TeamId::Red),
            })
            .collect();
        
        // Calculate team info
        let team_player_count = player_views.iter()
            .filter(|p| p.team == ai_team)
            .count();
        
        let team_mech_count = mech_views.iter()
            .filter(|m| m.team == ai_team)
            .count();
        
        let mut total_resources = std::collections::HashMap::new();
        for mech in mech_views.iter().filter(|m| m.team == ai_team) {
            for (resource_type, count) in &mech.resource_inventory {
                *total_resources.entry(*resource_type).or_insert(0) += count;
            }
        }
        
        Some(GameView {
            tick: 0, // TODO: Track tick count
            players: player_views,
            mechs: mech_views,
            resources: resource_views,
            projectiles: projectile_views,
            team_info: TeamInfo {
                team_id: ai_team,
                player_count: team_player_count,
                mech_count: team_mech_count,
                total_resources,
            },
        })
    } else {
        None
    }
}