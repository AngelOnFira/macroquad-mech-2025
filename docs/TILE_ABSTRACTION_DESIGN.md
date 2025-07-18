# Tile Abstraction Design (OBSOLETE)

## Status: SUPERSEDED by Hybrid Tile-Entity System

**Note**: This document describes an earlier proposed trait-based tile system that has been superseded by the hybrid tile-entity approach. For the current implementation, see:
- [HYBRID_TILE_ENTITY_SYSTEM.md](./HYBRID_TILE_ENTITY_SYSTEM.md) - Current implementation guide
- [MECH_INTERIOR_DESIGN.md](./MECH_INTERIOR_DESIGN.md) - Mech interior implementation

## Why This Design Was Replaced

The trait-based approach described below was replaced with a hybrid system that better balances performance and flexibility:

1. **Performance**: Trait objects require dynamic dispatch, which is slower than enum matching for simple tiles
2. **Complexity**: Most tiles (walls, floors) don't need the full flexibility of the trait system
3. **Memory**: Boxing every tile increases memory usage significantly
4. **Simplicity**: The hybrid approach is easier to understand and maintain

## Current Hybrid Approach Summary

Instead of everything being a trait object, we now use:
- **Static Tiles**: Simple enum for walls, floors, windows (90% of tiles)
- **Entity References**: Complex objects (stations, turrets) use ECS with UUID references
- **Best of Both**: Performance for simple tiles, flexibility for complex objects

---

# Original Trait-Based Design (Historical Reference)

## Original Problems This Tried to Solve
1. **Dual Tile Systems**: `WorldTile` and `MechInteriorTile` were separate
2. **Hard-coded Behaviors**: Tile interactions scattered in code
3. **Limited Extensibility**: Adding tiles required many changes
4. **No Vision System**: Tiles didn't affect visibility
5. **Static Properties**: No dynamic tile state

## Original Proposed Trait Design

```rust
// This was the proposed base trait for all tiles
pub trait Tile: Send + Sync {
    fn tile_type(&self) -> TileType;
    fn get_state(&self) -> TileState;
    fn get_render_info(&self) -> RenderInfo;
    fn is_walkable(&self) -> bool;
    fn blocks_vision(&self) -> bool;
    fn on_interact(&mut self, actor: EntityId, world: &mut World) -> Vec<GameEvent>;
    fn update(&mut self, delta: f32, world: &World) -> Vec<GameEvent>;
    // ... many more methods
}
```

## Why This Approach Was Problematic

1. **Over-engineering**: Most tiles don't need all these methods
2. **Performance**: Virtual dispatch for every tile check
3. **Memory overhead**: Each tile needs heap allocation
4. **Complexity**: Too many concepts for simple tiles

## Lessons Learned

The hybrid approach we implemented provides:
- Simple tiles remain simple (just data)
- Complex objects get full ECS flexibility
- Better performance for the common case
- Easier to understand and modify

For details on the actual implementation, please refer to the current documentation files linked above.