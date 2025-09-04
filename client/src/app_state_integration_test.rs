#[cfg(test)]
mod integration_tests {
    use crate::app_state::AppState;
    use crate::game_state::GameState;
    use std::sync::{Arc, Mutex};

    /// Integration test for main.rs changes
    /// Tests that AppState integrates correctly with the existing game loop architecture
    #[test]
    fn test_app_state_integration() {
        // Test 1: AppState creation and initial state
        let app_state = Arc::new(Mutex::new(AppState::new()));
        let shared_game_state = Arc::new(Mutex::new(GameState::new()));
        
        // Should start in MainMenu
        {
            let app = app_state.lock().unwrap();
            assert!(app.is_in_menu());
            assert!(!app.is_in_game());
            assert!(!app.is_paused());
            assert_eq!(app.state_name(), "MainMenu");
        }

        // Test 2: Transition to Game state
        {
            let mut app = app_state.lock().unwrap();
            let new_game_state = GameState::new();
            app.transition_to_game(new_game_state).unwrap();
            
            assert!(app.is_in_game());
            assert!(!app.is_in_menu());
            assert!(!app.is_paused());
            assert_eq!(app.state_name(), "Game");
            assert!(app.game_state().is_some());
        }

        // Test 3: Game state synchronization pattern
        // Simulate the pattern used in main.rs where we sync between AppState and shared_game_state
        {
            let app = app_state.lock().unwrap();
            if app.is_in_game() {
                drop(app);
                let _shared_game = shared_game_state.lock().unwrap();
                // This simulates the locking pattern from main.rs - testing that it works without deadlock
                // No assertion needed - just testing that the locking pattern works
            }
        }

        // Test 4: Pause and resume functionality
        {
            let mut app = app_state.lock().unwrap();
            // Should be able to pause from Game state
            app.pause_game().unwrap();
            assert!(app.is_paused());
            assert!(!app.is_in_game());
            assert!(!app.is_in_menu());
            assert_eq!(app.state_name(), "Paused");
            
            // Should be able to resume
            app.resume_game().unwrap();
            assert!(app.is_in_game());
            assert!(!app.is_paused());
        }

        // Test 5: Settings transition pattern
        {
            let mut app = app_state.lock().unwrap();
            app.transition_to_main_menu().unwrap();
            assert!(app.is_in_menu());
            
            app.transition_to_settings().unwrap();
            assert!(app.is_in_menu());
            assert_eq!(app.state_name(), "Settings");
            
            app.return_from_settings().unwrap();
            assert_eq!(app.state_name(), "MainMenu");
        }
    }

    #[test]
    fn test_state_transition_errors() {
        let mut app_state = AppState::new();
        
        // Cannot pause when not in game
        assert!(app_state.pause_game().is_err());
        
        // Cannot resume when not paused
        assert!(app_state.resume_game().is_err());
        
        // Cannot return from settings when not in settings
        assert!(app_state.return_from_settings().is_err());
    }

    #[test]
    fn test_game_state_access_patterns() {
        let mut app_state = AppState::new();
        
        // No game state access when in menu
        assert!(app_state.game_state().is_none());
        assert!(app_state.game_state_mut().is_none());
        
        // Game state available when in game
        let game_state = GameState::new();
        app_state.transition_to_game(game_state).unwrap();
        
        // Test immutable access
        assert!(app_state.game_state().is_some());
        
        // Test mutable access (pattern used in main.rs)
        if let Some(_game) = app_state.game_state_mut() {
            // Just test that we can get mutable access - don't call update() due to macroquad dependencies
        }
        
        // Game state still available when paused
        app_state.pause_game().unwrap();
        assert!(app_state.game_state().is_some());
        assert!(app_state.game_state_mut().is_some());
    }

    #[test]
    fn test_concurrent_access_pattern() {
        // Test the concurrent access pattern used in main.rs
        let app_state = Arc::new(Mutex::new(AppState::new()));
        let shared_game_state = Arc::new(Mutex::new(GameState::new()));
        
        // Start game
        {
            let mut app = app_state.lock().unwrap();
            app.transition_to_game(GameState::new()).unwrap();
            drop(app);
            *shared_game_state.lock().unwrap() = GameState::new();
        }
        
        // Simulate the pattern from main.rs update loop
        {
            let app = app_state.lock().unwrap();
            if app.is_in_game() {
                drop(app); // Release app lock before acquiring game lock
                let _game = shared_game_state.lock().unwrap();
                // Simulate accessing game state - just testing locking pattern works
            }
        }
        
        // Simulate the pattern from rendering
        {
            let app = app_state.lock().unwrap();
            if app.is_in_game() || app.is_paused() {
                drop(app);
                let _game = shared_game_state.lock().unwrap();
                // Simulated rendering access - should work
            }
        }
    }
}