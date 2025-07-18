# Tile Abstraction Design

## Current State Analysis

### Problems with Current System
1. **Dual Tile Systems**: `WorldTile` and `MechInteriorTile` are separate enums with different properties
2. **Hard-coded Behaviors**: Tile interactions are scattered throughout movement code
3. **Limited Extensibility**: Adding new tile types requires modifying multiple locations
4. **No Vision System**: Tiles don't affect visibility or line of sight
5. **Static Properties**: Tiles can't have dynamic state or respond to events

## Proposed Tile Architecture

### Core Trait Design

```rust
// Base tile trait that all tiles implement
pub trait Tile: Send + Sync {
    // Identity
    fn tile_type(&self) -> TileType;
    fn get_state(&self) -> TileState;
    
    // Rendering
    fn get_render_info(&self) -> RenderInfo;
    fn get_light_properties(&self) -> LightProperties;
    
    // Movement and Collision  
    fn is_walkable(&self) -> bool;
    fn get_movement_cost(&self) -> f32; // 1.0 = normal, higher = slower
    fn blocks_projectiles(&self) -> bool;
    
    // Vision
    fn blocks_vision(&self) -> bool;
    fn vision_attenuation(&self) -> f32; // 0.0 = transparent, 1.0 = opaque
    
    // Interaction
    fn can_interact(&self) -> bool;
    fn get_interaction_prompt(&self) -> Option<String>;
    fn on_interact(&mut self, actor: EntityId, world: &mut World) -> Vec<GameEvent>;
    
    // Events
    fn on_enter(&mut self, actor: EntityId, world: &World) -> Vec<GameEvent>;
    fn on_exit(&mut self, actor: EntityId, world: &World) -> Vec<GameEvent>;
    fn on_step(&mut self, actor: EntityId, world: &World) -> Vec<GameEvent>;
    
    // Updates
    fn update(&mut self, delta: f32, world: &World) -> Vec<GameEvent>;
    fn on_neighbor_changed(&mut self, direction: Direction, neighbor: Option<&TileType>);
}

// Tile state that can change at runtime
pub struct TileState {
    pub damage: f32,
    pub powered: bool,
    pub custom_data: HashMap<String, Value>,
}

// Rendering information
pub struct RenderInfo {
    pub base_color: Color,
    pub texture_id: Option<TextureId>,
    pub render_layer: RenderLayer,
    pub opacity: f32,
    pub animation_state: Option<AnimationState>,
}

// Light interaction properties
pub struct LightProperties {
    pub blocks_light: bool,
    pub emits_light: Option<LightEmission>,
    pub transparency: f32,
}
```

### Concrete Tile Examples

```rust
// Window tile that allows vision but blocks movement
pub struct WindowTile {
    facing: Direction,
    tint: Color,
    broken: bool,
}

impl Tile for WindowTile {
    fn is_walkable(&self) -> bool { false }
    fn blocks_vision(&self) -> bool { false }
    fn vision_attenuation(&self) -> f32 { 
        if self.broken { 0.0 } else { 0.2 }
    }
    fn blocks_projectiles(&self) -> bool { !self.broken }
    
    fn on_interact(&mut self, actor: EntityId, world: &mut World) -> Vec<GameEvent> {
        if self.broken {
            vec![]
        } else {
            vec![GameEvent::ShowMessage {
                actor,
                message: "The window is reinforced glass.".to_string()
            }]
        }
    }
}

// Transition tile for stairs/entrances
pub struct TransitionTile {
    transition_type: TransitionType,
    target_location: LocationSpecifier,
    progress_required: f32,
    current_progress: HashMap<EntityId, f32>,
}

impl Tile for TransitionTile {
    fn on_enter(&mut self, actor: EntityId, world: &World) -> Vec<GameEvent> {
        self.current_progress.insert(actor, 0.0);
        vec![GameEvent::StartTransition {
            actor,
            transition_type: self.transition_type,
            target: self.target_location.clone(),
        }]
    }
    
    fn on_step(&mut self, actor: EntityId, world: &World) -> Vec<GameEvent> {
        if let Some(progress) = self.current_progress.get_mut(&actor) {
            *progress += world.get_delta_time();
            if *progress >= self.progress_required {
                vec![GameEvent::CompleteTransition { actor }]
            } else {
                vec![GameEvent::UpdateTransition {
                    actor,
                    progress: *progress / self.progress_required,
                }]
            }
        } else {
            vec![]
        }
    }
}
```

### Tile Container Structure

```rust
// Manages all tiles in the game world
pub struct TileMap {
    // Static world tiles
    world_tiles: HashMap<TilePos, Box<dyn Tile>>,
    
    // Mech interior tiles (keyed by mech ID and local position)
    mech_tiles: HashMap<Uuid, HashMap<(u8, TilePos), Box<dyn Tile>>>,
    
    // Spatial indices for fast lookups
    tile_spatial_index: SpatialIndex<TilePos>,
    
    // Tile update queue
    tiles_needing_update: HashSet<TilePos>,
}

impl TileMap {
    // Get tile at world position, accounting for mechs
    pub fn get_tile_at(&self, world_pos: WorldPos) -> Option<&dyn Tile> {
        // First check if we're inside a mech
        if let Some((mech_id, local_pos)) = self.world_to_mech_local(world_pos) {
            self.mech_tiles.get(&mech_id)
                .and_then(|mech| mech.get(&local_pos))
                .map(|t| t.as_ref())
        } else {
            // Otherwise check world tiles
            let tile_pos = world_pos.to_tile_pos();
            self.world_tiles.get(&tile_pos).map(|t| t.as_ref())
        }
    }
    
    // Update all dynamic tiles
    pub fn update(&mut self, delta: f32, world: &World) -> Vec<GameEvent> {
        let mut events = Vec::new();
        
        for tile_pos in &self.tiles_needing_update {
            if let Some(tile) = self.world_tiles.get_mut(tile_pos) {
                events.extend(tile.update(delta, world));
            }
        }
        
        events
    }
}
```

### Vision System Integration

```rust
pub struct VisionSystem {
    // Cached visibility data per viewer
    visibility_cache: HashMap<EntityId, VisibilityData>,
}

pub struct VisibilityData {
    visible_tiles: HashSet<TilePos>,
    light_levels: HashMap<TilePos, f32>,
    last_update_pos: WorldPos,
}

impl VisionSystem {
    pub fn calculate_visibility(
        &mut self,
        viewer_pos: WorldPos,
        max_range: f32,
        tile_map: &TileMap,
    ) -> &VisibilityData {
        // Raycast from viewer position
        let mut visible = HashSet::new();
        let mut light_levels = HashMap::new();
        
        // Cast rays in all directions
        for angle in 0..360 {
            let (dx, dy) = angle_to_direction(angle as f32);
            let mut distance = 0.0;
            let mut light_remaining = 1.0;
            
            while distance < max_range && light_remaining > 0.01 {
                let check_pos = WorldPos {
                    x: viewer_pos.x + dx * distance,
                    y: viewer_pos.y + dy * distance,
                };
                
                if let Some(tile) = tile_map.get_tile_at(check_pos) {
                    let tile_pos = check_pos.to_tile_pos();
                    visible.insert(tile_pos);
                    light_levels.insert(tile_pos, light_remaining);
                    
                    // Reduce light based on tile properties
                    light_remaining *= 1.0 - tile.vision_attenuation();
                    
                    if tile.blocks_vision() {
                        break;
                    }
                }
                
                distance += 0.5; // Step size
            }
        }
        
        // Cache and return
        self.visibility_cache.insert(viewer_id, VisibilityData {
            visible_tiles: visible,
            light_levels,
            last_update_pos: viewer_pos,
        })
    }
}
```

## Benefits of This Design

1. **Extensibility**: New tile types just implement the trait
2. **Dynamic Behavior**: Tiles can have state and respond to events
3. **Unified System**: One tile system for everything
4. **Performance**: Spatial indexing and caching for efficiency
5. **Moddability**: Tiles could be loaded from data files
6. **Clear Separation**: Rendering, physics, and gameplay logic are separate

## Migration Path

1. Create new trait-based tile system alongside existing
2. Implement adapters for current tile types
3. Gradually migrate systems to use new tiles
4. Remove old tile enums once migration complete

## Open Questions

1. Should tiles be able to modify other tiles directly?
2. How do we handle tile persistence/serialization?
3. Should tiles have built-in networking support?
4. How complex should the event system be?
5. Should we support tile "components" for mix-and-match behaviors?