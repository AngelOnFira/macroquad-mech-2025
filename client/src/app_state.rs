use crate::game_state::GameState;

/// Errors that can occur during state transitions
#[derive(Debug, Clone)]
pub enum StateError {
    InvalidTransition { from: String, to: String },
    GameStateNotAvailable,
    MenuNotInitialized,
}

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateError::InvalidTransition { from, to } => {
                write!(f, "Invalid state transition from {} to {}", from, to)
            }
            StateError::GameStateNotAvailable => {
                write!(f, "Game state is not available")
            }
            StateError::MenuNotInitialized => {
                write!(f, "Menu is not initialized")
            }
        }
    }
}

impl std::error::Error for StateError {}

/// Application state enum that wraps GameState and adds menu states
pub enum AppState {
    MainMenu,
    Settings { previous_state: Box<AppState> },
    Game(GameState),
    Paused(GameState),
}

impl AppState {
    /// Create a new AppState starting with MainMenu
    pub fn new() -> Self {
        AppState::MainMenu
    }

    /// Create a new AppState starting with Game state
    pub fn new_with_game(game_state: GameState) -> Self {
        AppState::Game(game_state)
    }

    /// Get the current state name as string for debugging/logging
    pub fn state_name(&self) -> &'static str {
        match self {
            AppState::MainMenu => "MainMenu",
            AppState::Settings { .. } => "Settings",
            AppState::Game(_) => "Game",
            AppState::Paused(_) => "Paused",
        }
    }

    /// Transition to game state with a new GameState
    pub fn transition_to_game(&mut self, game_state: GameState) -> Result<(), StateError> {
        match self {
            AppState::MainMenu => {
                *self = AppState::Game(game_state);
                Ok(())
            }
            AppState::Settings { .. } => {
                *self = AppState::Game(game_state);
                Ok(())
            }
            AppState::Game(_) => {
                // Already in game, replace the game state
                *self = AppState::Game(game_state);
                Ok(())
            }
            AppState::Paused(_) => {
                *self = AppState::Game(game_state);
                Ok(())
            }
        }
    }

    /// Transition to main menu
    pub fn transition_to_main_menu(&mut self) -> Result<(), StateError> {
        *self = AppState::MainMenu;
        Ok(())
    }

    /// Transition to settings, remembering the previous state
    pub fn transition_to_settings(&mut self) -> Result<(), StateError> {
        let previous = std::mem::replace(self, AppState::MainMenu);
        *self = AppState::Settings { 
            previous_state: Box::new(previous) 
        };
        Ok(())
    }

    /// Return from settings to the previous state
    pub fn return_from_settings(&mut self) -> Result<(), StateError> {
        match self {
            AppState::Settings { previous_state } => {
                let previous = std::mem::replace(previous_state.as_mut(), AppState::MainMenu);
                *self = previous;
                Ok(())
            }
            _ => Err(StateError::InvalidTransition {
                from: self.state_name().to_string(),
                to: "PreviousFromSettings".to_string(),
            }),
        }
    }

    /// Pause the game if currently in game state
    pub fn pause_game(&mut self) -> Result<(), StateError> {
        match self {
            AppState::Game(game_state) => {
                let game_state = std::mem::replace(game_state, GameState::new());
                *self = AppState::Paused(game_state);
                Ok(())
            }
            _ => Err(StateError::InvalidTransition {
                from: self.state_name().to_string(),
                to: "Paused".to_string(),
            }),
        }
    }

    /// Resume the game if currently paused
    pub fn resume_game(&mut self) -> Result<(), StateError> {
        match self {
            AppState::Paused(game_state) => {
                let game_state = std::mem::replace(game_state, GameState::new());
                *self = AppState::Game(game_state);
                Ok(())
            }
            _ => Err(StateError::InvalidTransition {
                from: self.state_name().to_string(),
                to: "Game".to_string(),
            }),
        }
    }

    /// Get a reference to the game state if available
    pub fn game_state(&self) -> Option<&GameState> {
        match self {
            AppState::Game(game_state) | AppState::Paused(game_state) => Some(game_state),
            _ => None,
        }
    }

    /// Get a mutable reference to the game state if available
    pub fn game_state_mut(&mut self) -> Option<&mut GameState> {
        match self {
            AppState::Game(game_state) | AppState::Paused(game_state) => Some(game_state),
            _ => None,
        }
    }

    /// Check if we're currently in the game (playing, not paused)
    pub fn is_in_game(&self) -> bool {
        matches!(self, AppState::Game(_))
    }

    /// Check if we're in any menu state
    pub fn is_in_menu(&self) -> bool {
        matches!(self, AppState::MainMenu | AppState::Settings { .. })
    }

    /// Check if the game is paused
    pub fn is_paused(&self) -> bool {
        matches!(self, AppState::Paused(_))
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_app_state() {
        let app_state = AppState::new();
        assert!(matches!(app_state, AppState::MainMenu));
        assert_eq!(app_state.state_name(), "MainMenu");
        assert!(app_state.is_in_menu());
        assert!(!app_state.is_in_game());
        assert!(!app_state.is_paused());
    }

    #[test]
    fn test_transition_to_game() {
        let mut app_state = AppState::new();
        let game_state = GameState::new();
        
        app_state.transition_to_game(game_state).unwrap();
        assert!(matches!(app_state, AppState::Game(_)));
        assert!(app_state.is_in_game());
        assert!(!app_state.is_in_menu());
        assert!(app_state.game_state().is_some());
    }

    #[test]
    fn test_transition_to_settings() {
        let mut app_state = AppState::new();
        
        app_state.transition_to_settings().unwrap();
        assert!(matches!(app_state, AppState::Settings { .. }));
        assert!(app_state.is_in_menu());
        assert!(!app_state.is_in_game());
    }

    #[test]
    fn test_return_from_settings() {
        let mut app_state = AppState::new();
        
        // Go to settings from main menu
        app_state.transition_to_settings().unwrap();
        assert!(matches!(app_state, AppState::Settings { .. }));
        
        // Return from settings
        app_state.return_from_settings().unwrap();
        assert!(matches!(app_state, AppState::MainMenu));
    }

    #[test]
    fn test_pause_and_resume_game() {
        let mut app_state = AppState::new();
        let game_state = GameState::new();
        
        // Start game
        app_state.transition_to_game(game_state).unwrap();
        assert!(app_state.is_in_game());
        
        // Pause game
        app_state.pause_game().unwrap();
        assert!(matches!(app_state, AppState::Paused(_)));
        assert!(app_state.is_paused());
        assert!(app_state.game_state().is_some());
        
        // Resume game
        app_state.resume_game().unwrap();
        assert!(matches!(app_state, AppState::Game(_)));
        assert!(app_state.is_in_game());
    }

    #[test]
    fn test_invalid_transitions() {
        let mut app_state = AppState::new();
        
        // Can't pause when not in game
        assert!(app_state.pause_game().is_err());
        
        // Can't resume when not paused
        assert!(app_state.resume_game().is_err());
        
        // Can't return from settings when not in settings
        assert!(app_state.return_from_settings().is_err());
    }

    #[test]
    fn test_game_state_access() {
        let mut app_state = AppState::new();
        
        // No game state when in menu
        assert!(app_state.game_state().is_none());
        assert!(app_state.game_state_mut().is_none());
        
        // Game state available when in game
        let game_state = GameState::new();
        app_state.transition_to_game(game_state).unwrap();
        assert!(app_state.game_state().is_some());
        assert!(app_state.game_state_mut().is_some());
        
        // Game state available when paused
        app_state.pause_game().unwrap();
        assert!(app_state.game_state().is_some());
        assert!(app_state.game_state_mut().is_some());
    }
}