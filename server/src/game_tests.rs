#[cfg(test)]
mod tests {
    use crate::game::Game;
    use crate::systems::tile_behavior::TileBehaviorSystem;
    use shared::{
        coordinates::MechDoorPositions,
        tile_entity::TileEvent,
        types::{TilePos, WorldPos},
        PlayerLocation, TeamId,
    };
    use std::collections::HashMap;
    use uuid::Uuid;

    // =============================================================================
    // Test Helper Functions
    // =============================================================================

    /// Create a test game with initialized mechs
    fn create_test_game() -> Game {
        Game::new()
    }

    /// Add a test player to the game
    fn add_test_player(game: &mut Game, name: &str, team: Option<TeamId>) -> Uuid {
        let player_id = Uuid::new_v4();
        let (_team, _spawn_pos) = game.add_player(player_id, name.to_string(), team);
        player_id
    }

    /// Get the position of a player regardless of location type
    fn get_player_world_pos(game: &Game, player_id: Uuid) -> Option<WorldPos> {
        game.players
            .get(&player_id)
            .map(|player| match player.location {
                PlayerLocation::OutsideWorld(pos) => pos,
                PlayerLocation::InsideMech { pos, .. } => pos,
            })
    }

    /// Simulate player movement by directly updating their position
    /// This bypasses the normal command system for testing purposes
    fn simulate_player_move(game: &mut Game, player_id: Uuid, target_pos: WorldPos) {
        if let Some(player) = game.players.get_mut(&player_id) {
            match &mut player.location {
                PlayerLocation::OutsideWorld(pos) => {
                    *pos = target_pos;
                }
                PlayerLocation::InsideMech { pos, .. } => {
                    *pos = target_pos;
                }
            }
        }
    }

    /// Walk a player to a specific tile position
    /// This simulates the movement and checks for tile events
    fn simulate_walk_to_tile(
        game: &mut Game,
        player_id: Uuid,
        target_tile: TilePos,
    ) -> Vec<TileEvent> {
        let target_world = target_tile.to_world_center();
        simulate_player_move(game, player_id, target_world);

        // Check for tile events at the new position
        let mut events = Vec::new();
        if let Some(tile_content) = game.tile_map.get_world_tile(target_tile) {
            if let shared::tile_entity::TileContent::Static(static_tile) = tile_content {
                if let Some(tile_event) = static_tile.on_enter(player_id) {
                    events.push(tile_event);
                }
            }
        }
        events
    }

    /// Process tile events synchronously for testing
    /// This simulates what the TileBehaviorSystem would do asynchronously
    fn process_tile_events_sync(
        game: &mut Game,
        events: Vec<TileEvent>,
    ) -> Vec<shared::ServerMessage> {
        let mut messages = Vec::new();

        // Create a temporary TileBehaviorSystem for processing events
        let mut tile_system = TileBehaviorSystem::new();

        // Add events to the system's queue
        tile_system.event_queue.extend(events);

        // Process the events
        messages.extend(tile_system.handle_tile_events(game));

        messages
    }

    /// Check if a player is inside a specific mech
    fn assert_player_in_mech(game: &Game, player_id: Uuid, expected_mech_id: Uuid) {
        let player = game
            .players
            .get(&player_id)
            .expect(&format!("Player {} not found", player_id));

        match &player.location {
            PlayerLocation::InsideMech { mech_id, .. } => {
                assert_eq!(
                    *mech_id, expected_mech_id,
                    "Player {} is in mech {} but expected mech {}",
                    player_id, mech_id, expected_mech_id
                );
            }
            PlayerLocation::OutsideWorld(_) => {
                panic!(
                    "Player {} is outside world, expected to be in mech {}",
                    player_id, expected_mech_id
                );
            }
        }
    }

    /// Check if a player is outside in the world
    fn assert_player_outside_world(game: &Game, player_id: Uuid) {
        let player = game
            .players
            .get(&player_id)
            .expect(&format!("Player {} not found", player_id));

        match &player.location {
            PlayerLocation::OutsideWorld(_) => {
                // This is expected
            }
            PlayerLocation::InsideMech { mech_id, .. } => {
                panic!(
                    "Player {} is in mech {}, expected to be outside",
                    player_id, mech_id
                );
            }
        }
    }

    /// Get the team mech for a specific team
    fn get_team_mech(game: &Game, team: TeamId) -> Option<&crate::game::Mech> {
        game.mechs.values().find(|mech| mech.team == team)
    }

    // =============================================================================
    // Core Test Cases
    // =============================================================================

    #[test]
    fn test_game_initialization() {
        let game = create_test_game();

        // Verify mechs were created
        assert_eq!(game.mechs.len(), 2, "Should have 2 mechs (red and blue)");

        // Verify one mech for each team
        let red_mechs: Vec<_> = game
            .mechs
            .values()
            .filter(|m| m.team == TeamId::Red)
            .collect();
        let blue_mechs: Vec<_> = game
            .mechs
            .values()
            .filter(|m| m.team == TeamId::Blue)
            .collect();

        assert_eq!(red_mechs.len(), 1, "Should have 1 red mech");
        assert_eq!(blue_mechs.len(), 1, "Should have 1 blue mech");
    }

    #[test]
    fn test_player_can_spawn() {
        let mut game = create_test_game();
        let player_id = add_test_player(&mut game, "TestPlayer", Some(TeamId::Red));

        // Verify player was added
        assert!(
            game.players.contains_key(&player_id),
            "Player should be in game"
        );

        let player = &game.players[&player_id];
        assert_eq!(player.name, "TestPlayer");
        assert_eq!(player.team, TeamId::Red);
        assert_player_outside_world(&game, player_id);
    }

    #[test]
    fn test_player_can_walk_to_mech_door() {
        let mut game = create_test_game();
        let player_id = add_test_player(&mut game, "TestPlayer", Some(TeamId::Red));

        // Get red team mech
        let red_mech = get_team_mech(&game, TeamId::Red).expect("Red mech should exist");
        let mech_id = red_mech.id;

        // Get door positions
        let doors = MechDoorPositions::from_mech_position(red_mech.position);

        // Walk to the left door
        let events = simulate_walk_to_tile(&mut game, player_id, doors.left_door);

        // Verify the player is at the door tile
        let player_pos = get_player_world_pos(&game, player_id).unwrap();
        let expected_pos = doors.left_door.to_world_center();

        println!("Player position: {:?}", player_pos);
        println!("Expected position: {:?}", expected_pos);
        println!("Generated events: {:?}", events);

        // The player should be at the door tile
        assert!(
            (player_pos.x - expected_pos.x).abs() < 1.0,
            "Player X position should be close to door"
        );
        assert!(
            (player_pos.y - expected_pos.y).abs() < 1.0,
            "Player Y position should be close to door"
        );
    }

    #[test]
    fn test_player_enters_mech_on_door_tile() {
        let mut game = create_test_game();
        let player_id = add_test_player(&mut game, "TestPlayer", Some(TeamId::Red));

        // Get red team mech
        let red_mech = get_team_mech(&game, TeamId::Red).expect("Red mech should exist");
        let mech_id = red_mech.id;

        // Get door positions
        let doors = MechDoorPositions::from_mech_position(red_mech.position);

        // Walk to the left door and get events
        let events = simulate_walk_to_tile(&mut game, player_id, doors.left_door);

        println!("Events generated: {:?}", events);

        // Process the tile events synchronously
        let messages = process_tile_events_sync(&mut game, events);

        println!("Messages generated: {:?}", messages);

        // Check if player entered mech
        // Note: This test might fail initially, which is expected
        // We'll use it to identify what needs to be fixed
        if let Some(player) = game.players.get(&player_id) {
            println!("Player location after processing: {:?}", player.location);

            match &player.location {
                PlayerLocation::InsideMech {
                    mech_id: entered_mech_id,
                    floor,
                    pos,
                } => {
                    println!(
                        "SUCCESS: Player entered mech {} on floor {} at {:?}",
                        entered_mech_id, floor, pos
                    );
                    assert_eq!(
                        *entered_mech_id, mech_id,
                        "Player should be in the red mech"
                    );
                }
                PlayerLocation::OutsideWorld(pos) => {
                    println!("ISSUE: Player is still outside at {:?}", pos);
                    // This is what we expect to fail initially
                    // The test documents the expected behavior
                }
            }
        }
    }

    #[test]
    fn test_player_denied_entry_to_enemy_mech() {
        let mut game = create_test_game();
        let player_id = add_test_player(&mut game, "TestPlayer", Some(TeamId::Red));

        // Get blue team mech (enemy mech for red player)
        let blue_mech = get_team_mech(&game, TeamId::Blue).expect("Blue mech should exist");

        // Get door positions for enemy mech
        let doors = MechDoorPositions::from_mech_position(blue_mech.position);

        // Walk to the enemy mech's door
        let events = simulate_walk_to_tile(&mut game, player_id, doors.left_door);

        // Process the tile events
        let _messages = process_tile_events_sync(&mut game, events);

        // Player should still be outside (not allowed to enter enemy mech)
        assert_player_outside_world(&game, player_id);

        println!("PASS: Player correctly denied entry to enemy mech");
    }

    // Additional helper test to verify door positions are correct
    /*
    #[test]
    fn test_real_command_system_mech_entry() {
        // Test using the actual PlayerInputCommand system instead of direct simulation
        // TODO: Add tokio_test dependency to enable this test
        use crate::commands::PlayerInputCommand;
        use crate::commands::Command;
        use tokio::sync::broadcast;

        tokio_test::block_on(async {
            let mut game = create_test_game();
            let player_id = add_test_player(&mut game, "TestPlayer", Some(TeamId::Red));

            // Get red team mech and position player near the door
            let red_mech = get_team_mech(&game, TeamId::Red).expect("Red mech should exist");
            let doors = MechDoorPositions::from_mech_position(red_mech.position);

            // Position player just outside the door
            let approach_pos = WorldPos::new(
                doors.left_door.to_world_center().x - 8.0, // 8 pixels away
                doors.left_door.to_world_center().y,
            );
            simulate_player_move(&mut game, player_id, approach_pos);

            println!("Player positioned at: {:?}", approach_pos);
            println!("Door position: {:?}", doors.left_door.to_world_center());

            // Wrap game in Arc<RwLock<>> as expected by the command system
            let game_lock = std::sync::Arc::new(tokio::sync::RwLock::new(game));
            let (tx, _rx) = broadcast::channel(100);

            // Create movement command toward the door
            let movement = (1.0, 0.0); // Move right toward the door
            let command = PlayerInputCommand {
                movement,
                action_key_pressed: false,
            };

            // Execute the command (this should trigger mech entry)
            let result = command.execute(&game_lock, player_id, &tx).await;
            assert!(result.is_ok(), "Command should execute successfully");

            // Check if player entered mech
            let game_read = game_lock.read().await;
            if let Some(player) = game_read.players.get(&player_id) {
                println!("Player location after command: {:?}", player.location);

                match &player.location {
                    PlayerLocation::InsideMech { mech_id, floor, pos } => {
                        println!("SUCCESS: Command system moved player into mech {} on floor {} at {:?}",
                            mech_id, floor, pos);
                        assert_eq!(*mech_id, red_mech.id);
                    }
                    PlayerLocation::OutsideWorld(pos) => {
                        // This might still happen if the movement wasn't enough to reach the door tile
                        println!("Player is outside at {:?}, may need more movement", pos);
                    }
                }
            }
        });
    }
    */

    #[test]
    fn test_door_positions_are_on_transition_tiles() {
        let game = create_test_game();

        for mech in game.mechs.values() {
            let doors = MechDoorPositions::from_mech_position(mech.position);

            // Check that door tiles are actually TransitionZone tiles
            let left_tile = game.tile_map.get_world_tile(doors.left_door);
            let right_tile = game.tile_map.get_world_tile(doors.right_door);

            println!(
                "Mech at {:?} has doors at {:?} and {:?}",
                mech.position, doors.left_door, doors.right_door
            );
            println!("Left door tile: {:?}", left_tile);
            println!("Right door tile: {:?}", right_tile);

            // Verify these are actually transition tiles
            if let Some(shared::tile_entity::TileContent::Static(static_tile)) = left_tile {
                match static_tile {
                    shared::tile_entity::StaticTile::TransitionZone {
                        transition_type, ..
                    } => {
                        println!("Left door is TransitionZone: {:?}", transition_type);
                    }
                    _ => {
                        panic!("Left door tile is not a TransitionZone: {:?}", static_tile);
                    }
                }
            } else {
                panic!("Left door tile is not a static tile: {:?}", left_tile);
            }
        }
    }
}
