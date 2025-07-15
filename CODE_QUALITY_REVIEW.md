# Code Quality Review - Mech Battle Arena

This document outlines code quality issues found in the codebase and provides concrete suggestions for improvement.

## 1. Massive Code Duplication

### Issues
- Three separate network implementations with duplicated message handling
- Mech interior floor layouts created identically in server and client
- Player movement logic duplicated between inside/outside mech movement

### Suggestions
- **Create a unified network abstraction layer:**
  ```rust
  trait NetworkHandler {
      fn send_message(&self, msg: ClientMessage);
      fn poll_messages(&mut self) -> Vec<ServerMessage>;
  }
  ```
- **Move shared logic to the `shared` crate:**
  - Floor layout generation
  - Movement validation rules
  - Common game constants
- **Use code generation for message handlers:**
  ```rust
  #[derive(MessageHandler)]
  enum ServerMessage {
      #[handler(fn = "handle_player_moved")]
      PlayerMoved { player_id: Uuid, location: PlayerLocation },
  }
  ```

## 2. Poor Error Handling

### Issues
- Extensive use of `unwrap()` without proper error handling
- Network failures are logged but not handled
- No validation of client inputs

### Suggestions
- **Define a comprehensive error type:**
  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum GameError {
      #[error("Network error: {0}")]
      Network(#[from] NetworkError),
      #[error("Invalid player action: {0}")]
      InvalidAction(String),
      #[error("Game state error: {0}")]
      GameState(String),
  }
  ```
- **Use Result types consistently:**
  ```rust
  pub fn handle_player_input(&mut self, input: PlayerInput) -> Result<(), GameError> {
      validate_input(&input)?;
      self.apply_input(input)
  }
  ```
- **Implement input validation layer:**
  ```rust
  pub struct InputValidator {
      pub fn validate_movement(&self, movement: (f32, f32)) -> Result<(f32, f32), ValidationError> {
          // Clamp values, check for NaN, etc.
      }
  }
  ```

## 3. Monolithic Functions

### Issues
- Functions exceeding 100+ lines
- Deeply nested logic
- Multiple responsibilities per function

### Suggestions
- **Break down large functions using the Command pattern:**
  ```rust
  trait StationCommand {
      fn execute(&self, game: &mut Game, player_id: Uuid) -> Result<(), GameError>;
  }
  
  struct UpgradeLaserCommand { /* ... */ }
  struct FireWeaponCommand { /* ... */ }
  ```
- **Extract complex conditions into well-named functions:**
  ```rust
  fn can_player_enter_mech(&self, player: &Player, mech: &Mech) -> bool {
      self.is_player_near_door(player, mech) 
          && self.is_mech_accessible(mech, player.team)
          && !self.is_player_carrying_resource(player)
  }
  ```
- **Use state machines for complex flows:**
  ```rust
  enum PlayerState {
      Idle,
      Moving { velocity: Vec2 },
      OperatingStation { station_id: Uuid },
      InMechTransition { from: Location, to: Location },
  }
  ```

## 4. Architecture Problems

### Issues
- No proper state management
- Tight coupling between systems
- Mixed responsibilities

### Suggestions
- **Implement an Entity Component System (ECS):**
  ```rust
  // Using specs or bevy_ecs
  struct Position(WorldPos);
  struct Velocity(f32, f32);
  struct Health(u32);
  struct MechPilot { mech_id: Uuid }
  
  // Systems operate on components
  struct MovementSystem;
  impl System for MovementSystem {
      fn run(&mut self, (positions, velocities): (WriteStorage<Position>, ReadStorage<Velocity>)) {
          // Update positions based on velocities
      }
  }
  ```
- **Create clear system boundaries:**
  ```rust
  pub struct GameSystems {
      physics: PhysicsSystem,
      combat: CombatSystem,
      resource: ResourceSystem,
      network: NetworkSystem,
  }
  ```
- **Use the Repository pattern for data access:**
  ```rust
  trait MechRepository {
      fn get(&self, id: Uuid) -> Option<&Mech>;
      fn get_mut(&mut self, id: Uuid) -> Option<&mut Mech>;
      fn find_in_range(&self, pos: WorldPos, range: f32) -> Vec<&Mech>;
  }
  ```

## 5. Magic Numbers Everywhere

### Issues
- Hardcoded values throughout the codebase
- No central configuration
- Unclear what numbers represent

### Suggestions
- **Create a comprehensive constants module:**
  ```rust
  pub mod balance {
      pub const PLAYER_BASE_SPEED: f32 = 5.0; // tiles per second
      pub const MECH_BASE_SPEED: f32 = 2.0;
      pub const RESOURCE_PICKUP_RANGE: f32 = 1.5; // tiles
      pub const LADDER_INTERACTION_RANGE: f32 = 0.3; // tiles
      
      pub mod combat {
          pub const LASER_BASE_DAMAGE: u32 = 10;
          pub const LASER_DAMAGE_PER_LEVEL: u32 = 10;
          pub const PROJECTILE_SPEED: f32 = 300.0; // pixels per second
      }
  }
  ```
- **Use configuration files for tuning:**
  ```toml
  # config/game_balance.toml
  [movement]
  player_speed = 5.0
  mech_speed = 2.0
  
  [combat.laser]
  base_damage = 10
  damage_per_level = 10
  ```
- **Create typed wrappers for domain values:**
  ```rust
  #[derive(Debug, Clone, Copy)]
  pub struct TilesPerSecond(pub f32);
  
  #[derive(Debug, Clone, Copy)]
  pub struct HealthPoints(pub u32);
  ```

## 6. Performance Issues

### Issues
- O(n²) collision detection
- Frequent allocations
- No spatial partitioning

### Suggestions
- **Implement spatial partitioning:**
  ```rust
  pub struct SpatialGrid<T> {
      cells: HashMap<(i32, i32), Vec<T>>,
      cell_size: f32,
  }
  
  impl<T> SpatialGrid<T> {
      pub fn query_radius(&self, pos: WorldPos, radius: f32) -> Vec<&T> {
          // Only check nearby cells
      }
  }
  ```
- **Use object pools for frequently allocated objects:**
  ```rust
  pub struct ProjectilePool {
      available: Vec<Projectile>,
      active: HashMap<Uuid, Projectile>,
  }
  ```
- **Implement dirty flags for expensive calculations:**
  ```rust
  pub struct MechState {
      position: WorldPos,
      position_dirty: bool,
      cached_tile_pos: TilePos,
  }
  ```

## 7. Inconsistent Coordinate Systems

### Issues
- Mix of TilePos and WorldPos
- Unclear conversion patterns
- No documentation on usage

### Suggestions
- **Create a unified coordinate system:**
  ```rust
  pub trait Coordinate: Copy {
      fn to_world(&self) -> WorldPos;
      fn to_tile(&self) -> TilePos;
      fn to_screen(&self, camera: &Camera) -> ScreenPos;
  }
  ```
- **Use newtype pattern for coordinate types:**
  ```rust
  #[derive(Debug, Clone, Copy)]
  pub struct WorldPos(Vec2);
  
  #[derive(Debug, Clone, Copy)]
  pub struct TilePos(IVec2);
  
  #[derive(Debug, Clone, Copy)]
  pub struct ScreenPos(Vec2);
  ```
- **Provide clear conversion utilities:**
  ```rust
  pub mod coords {
      pub fn world_to_tile(world: WorldPos) -> TilePos {
          TilePos(IVec2::new(
              (world.0.x / TILE_SIZE).floor() as i32,
              (world.0.y / TILE_SIZE).floor() as i32,
          ))
      }
  }
  ```

## 8. Missing Core Features

### Issues
- No proper physics system
- No animation system
- No audio architecture

### Suggestions
- **Implement a proper physics engine:**
  ```rust
  pub struct PhysicsEngine {
      bodies: HashMap<Uuid, RigidBody>,
      constraints: Vec<Constraint>,
  }
  
  pub struct RigidBody {
      position: WorldPos,
      velocity: Vec2,
      acceleration: Vec2,
      mass: f32,
      drag: f32,
  }
  ```
- **Create an animation system:**
  ```rust
  pub struct AnimationSystem {
      animations: HashMap<String, Animation>,
      playing: HashMap<Uuid, PlayingAnimation>,
  }
  
  pub struct Animation {
      frames: Vec<Frame>,
      duration: f32,
      looping: bool,
  }
  ```
- **Design audio architecture:**
  ```rust
  pub trait AudioBackend {
      fn play_sound(&mut self, sound: &Sound, params: PlayParams);
      fn play_music(&mut self, music: &Music);
  }
  ```

## 9. Network Protocol Issues

### Issues
- No versioning
- No message validation
- Sending full state instead of deltas

### Suggestions
- **Implement protocol versioning:**
  ```rust
  #[derive(Serialize, Deserialize)]
  pub struct NetworkMessage {
      version: u32,
      #[serde(flatten)]
      content: MessageContent,
  }
  ```
- **Use delta compression:**
  ```rust
  pub struct StateDelta {
      tick: u64,
      changed_entities: Vec<EntityChange>,
      removed_entities: Vec<Uuid>,
  }
  ```
- **Add message validation:**
  ```rust
  impl Validate for ClientMessage {
      fn validate(&self) -> Result<(), ValidationError> {
          match self {
              ClientMessage::PlayerInput { movement, .. } => {
                  validate_movement_input(*movement)?;
              }
              // ... other validations
          }
          Ok(())
      }
  }
  ```

## 10. Code Organization Problems

### Issues
- Game logic scattered across files
- No clear separation between systems
- Hardcoded station logic

### Suggestions
- **Use a plugin architecture:**
  ```rust
  pub trait Plugin {
      fn build(&self, app: &mut App);
  }
  
  pub struct WeaponPlugin;
  impl Plugin for WeaponPlugin {
      fn build(&self, app: &mut App) {
          app.add_system(weapon_targeting_system)
             .add_system(projectile_movement_system)
             .add_system(damage_calculation_system);
      }
  }
  ```
- **Implement a registry pattern for stations:**
  ```rust
  pub struct StationRegistry {
      handlers: HashMap<StationType, Box<dyn StationHandler>>,
  }
  
  pub trait StationHandler {
      fn get_ui(&self) -> StationUI;
      fn handle_input(&self, input: StationInput, game: &mut Game) -> Result<(), GameError>;
      fn get_costs(&self) -> Vec<(ResourceType, u32)>;
  }
  ```
- **Use feature modules:**
  ```
  src/
    combat/
      mod.rs
      weapons.rs
      projectiles.rs
      damage.rs
    movement/
      mod.rs
      player.rs
      mech.rs
      pathfinding.rs
    stations/
      mod.rs
      weapon_station.rs
      engine_station.rs
      upgrade_station.rs
  ```

## 11. Testing & Documentation

### Issues
- No unit tests
- No integration tests
- Missing documentation

### Suggestions
- **Implement comprehensive testing:**
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      
      #[test]
      fn test_player_movement_bounds() {
          let mut game = Game::new();
          let player_id = game.add_player("Test", TeamId::Red);
          
          // Test movement stays within bounds
          game.move_player(player_id, (1000.0, 1000.0));
          let player = game.get_player(player_id).unwrap();
          assert!(player.position.x < ARENA_WIDTH);
      }
  }
  ```
- **Add integration tests:**
  ```rust
  #[tokio::test]
  async fn test_multiplayer_resource_pickup() {
      let mut server = TestServer::new().await;
      let client1 = TestClient::connect(&server).await;
      let client2 = TestClient::connect(&server).await;
      
      // Test resource competition
      server.spawn_resource_at(TilePos::new(50, 50));
      client1.move_to(TilePos::new(50, 50)).await;
      client2.move_to(TilePos::new(50, 50)).await;
      
      // Only one should get the resource
  }
  ```
- **Document with examples:**
  ```rust
  /// Handles player movement within the game world.
  /// 
  /// # Arguments
  /// * `movement` - Normalized movement vector (-1.0 to 1.0 for each axis)
  /// 
  /// # Example
  /// ```
  /// game.handle_player_movement(player_id, (0.0, -1.0)); // Move up
  /// game.handle_player_movement(player_id, (0.7, 0.7));  // Move diagonally
  /// ```
  pub fn handle_player_movement(&mut self, player_id: Uuid, movement: (f32, f32)) -> Result<(), GameError> {
  ```

## 12. Additional Architectural Improvements

### State Management
```rust
// Use a proper state machine
pub struct GameStateMachine {
    states: HashMap<TypeId, Box<dyn GameState>>,
    current: TypeId,
}

pub trait GameState {
    fn enter(&mut self, game: &mut Game);
    fn update(&mut self, game: &mut Game, dt: f32);
    fn exit(&mut self, game: &mut Game);
}
```

### Event System
```rust
pub enum GameEvent {
    PlayerMoved { player_id: Uuid, from: WorldPos, to: WorldPos },
    ResourceCollected { player_id: Uuid, resource_type: ResourceType },
    MechDestroyed { mech_id: Uuid, killer_id: Option<Uuid> },
}

pub struct EventBus {
    listeners: HashMap<TypeId, Vec<Box<dyn EventListener>>>,
}
```

### Resource Management
```rust
pub struct AssetManager {
    textures: HashMap<String, Texture>,
    sounds: HashMap<String, Sound>,
    configs: HashMap<String, Config>,
}

impl AssetManager {
    pub async fn load_all(&mut self) -> Result<(), AssetError> {
        // Load all game assets
    }
}
```

## Implementation Priority

1. **High Priority (Do First)**
   - Error handling improvements
   - Extract magic numbers to constants
   - Break down monolithic functions
   - Add input validation

2. **Medium Priority**
   - Implement spatial partitioning
   - Create unified coordinate system
   - Add basic tests
   - Improve code organization

3. **Low Priority (Long-term)**
   - Full ECS implementation
   - Plugin architecture
   - Advanced physics system
   - Comprehensive animation system

## Conclusion

These improvements would significantly enhance code maintainability, performance, and extensibility. Start with high-priority items that provide immediate benefits with minimal disruption to existing functionality.