# AI Implementation for Mech Battle Arena

## Overview

The AI system has been implemented as a separate crate that integrates with the game server. It provides intelligent computer-controlled players that can participate in the game alongside human players.

## Architecture

### AI Crate Structure

```
ai/
├── src/
│   ├── lib.rs              # Main AI manager and configuration
│   ├── interface.rs        # Core AI interface and types
│   ├── perception.rs       # Game state perception and analysis
│   ├── hats.rs            # Dynamic role system (hats)
│   ├── communication.rs    # Inter-AI messaging system
│   ├── utility.rs         # Utility AI implementation
│   ├── personality.rs     # AI personality traits
│   ├── decision.rs        # Decision making and command conversion
│   └── logging.rs         # Debug logging and visualization data
```

### Key Components

1. **AIManager**: Central coordinator for all AI players
   - Manages AI controllers
   - Handles communication between AIs
   - Logs decisions for debugging

2. **AIController Interface**: Abstract interface for different AI implementations
   - `perceive()`: Analyze game state
   - `decide()`: Make decisions based on perception
   - `get_debug_info()`: Provide debugging information

3. **Perception System**: Converts raw game state into AI-understandable information
   - Identifies threats and opportunities
   - Tracks team state and resources
   - Provides environmental awareness

4. **Hat System**: Dynamic role switching based on situation
   - Primary hats: Pilot, Gunner, Engineer, Scavenger, Scout, Defender
   - Reactive hats: UnderAttack, EmergencyRepair, ResourceRush, etc.
   - Special roles: Captain (for harder difficulties), Support

5. **Communication System**: Allows AIs to coordinate
   - Message types: Commands, Status updates, Requests, Intel, Coordination
   - Captain role can issue team-wide orders
   - Supports both broadcast and direct messages

6. **Personality System**: Different AI playstyles
   - Aggressive: Focus on combat
   - Defensive: Prioritize protection
   - Support: Resource gathering and repairs
   - Balanced: Adaptive behavior

7. **Decision System**: Converts AI decisions to game commands
   - Maps high-level actions to specific game inputs
   - Includes confidence levels
   - Supports multiple action types

## AI Types

### UtilityAI (Advanced)
- Scores all possible actions and picks the best
- Considers personality preferences
- Adapts to changing situations
- Used for difficulty > 0.7

### SimpleAI (Basic)
- Makes simpler, more predictable decisions
- Still uses hat system but with less sophistication
- Used for lower difficulties

## Integration with Server

The AI system is integrated as a game system in the server:

```rust
// server/src/systems/ai.rs
pub struct AISystem {
    ai_manager: AIManager,
    ai_players: HashMap<Uuid, AIPlayerInfo>,
}
```

### HTTP API

Add AI players via HTTP POST:
```bash
curl -X POST http://localhost:3030/ai/add \
  -H "Content-Type: application/json" \
  -d '{"difficulty": 0.5, "personality": "aggressive"}'
```

Response:
```json
{
  "ai_id": "uuid-here",
  "name": "AI_Hunter",
  "team": "Red"
}
```

## Features Implemented

✅ **Core AI System**
- Abstract interface for multiple AI implementations
- Utility AI with scoring system
- Simple AI for easier difficulties

✅ **Hat System**
- Dynamic role switching
- Context-aware hat selection
- Reactive hats for emergencies

✅ **Communication**
- Inter-AI messaging
- Captain role for team coordination
- Multiple message types

✅ **Personalities**
- Four distinct personality types
- Affects decision making
- Influences task preferences

✅ **Decision Logging**
- Logs all AI decisions
- Exports to JSON
- Performance metrics tracking

✅ **Server Integration**
- AI system as a game system
- HTTP endpoint for adding AIs
- Automatic team balancing

## Testing

Use the provided test script or just command:
```bash
just test-ai
# or
./test_ai.sh
```

This will add several AI players with different personalities and difficulties.

## Future Enhancements

### Pending Implementation
- [ ] Egui debug client for AI introspection
- [ ] Pause/step simulation for debugging
- [ ] Parallel AI processing
- [ ] Limited vision simulation
- [ ] Machine learning integration

### Debug Visualization
The system is prepared for debug visualization with:
- AI state snapshots
- Communication graphs
- Decision timelines
- Performance metrics

## Configuration

AI behavior can be configured through:
- Difficulty level (0.0 - 1.0)
- Personality selection
- Captain role enable/disable
- Update frequency
- Debug logging

## Performance Considerations

- AIs update at configurable frequency (default: 20 Hz)
- Efficient perception system with spatial queries
- Minimal allocation in hot paths
- Prepared for parallel processing