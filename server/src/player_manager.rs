use naia_server::UserKey;
use shared::types::TeamId;
use std::collections::HashMap;

pub struct PlayerManager {
    user_to_entity: HashMap<UserKey, u16>,
    entity_to_user: HashMap<u16, UserKey>,
    player_teams: HashMap<UserKey, TeamId>,
}

impl PlayerManager {
    pub fn new() -> Self {
        Self {
            user_to_entity: HashMap::new(),
            entity_to_user: HashMap::new(),
            player_teams: HashMap::new(),
        }
    }

    pub fn add_player(&mut self, user_key: UserKey, entity_id: u16, team: TeamId) {
        self.user_to_entity.insert(user_key, entity_id);
        self.entity_to_user.insert(entity_id, user_key);
        self.player_teams.insert(user_key, team);
    }

    pub fn remove_player(&mut self, user_key: &UserKey) {
        if let Some(entity_id) = self.user_to_entity.remove(user_key) {
            self.entity_to_user.remove(&entity_id);
        }
        self.player_teams.remove(user_key);
    }

    pub fn get_player_entity(&self, user_key: &UserKey) -> Option<u16> {
        self.user_to_entity.get(user_key).copied()
    }

    pub fn get_user_key(&self, entity_id: u16) -> Option<&UserKey> {
        self.entity_to_user.get(&entity_id)
    }

    pub fn get_player_team(&self, user_key: &UserKey) -> Option<TeamId> {
        self.player_teams.get(user_key).copied()
    }
}