# AI Debug Client

The AI Debug Client is an egui-based tool for introspecting and debugging AI behavior in real-time during gameplay.

## Features

### Real-Time AI Monitoring
- View all AI players and their current states
- See active "hats" (roles) and current actions
- Monitor confidence levels for decisions
- Track health and team status

### Communication Visualization
- View AI-to-AI messages
- See captain commands and team coordination
- Track message frequency and types
- Visual communication graph (text-based for now)

### Decision Timeline
- Chronological view of AI decisions
- See decision types and confidence levels
- Track reasoning for each action
- Identify patterns in AI behavior

### Performance Metrics
- Total decisions made
- Average decision time
- Decisions per second
- Message count
- Task success rate

### Simulation Controls
- Pause/resume simulation
- Step through decisions one at a time
- Adjust simulation speed (0.1x to 5x)
- Add/remove AI players on the fly

## Usage

### Starting the Debug Client

```bash
# Start the game server first
just dev

# In another terminal, start the debug client
just debug-ai
```

### Interface Overview

1. **Top Panel**: Connection status and simulation controls
   - Connect/disconnect button
   - Pause/resume simulation
   - Step button for frame-by-frame debugging
   - Speed slider

2. **Left Panel**: AI player list
   - Shows all AI players with their names
   - Click to select an AI for detailed view
   - Expandable details show team, health, hat, and confidence

3. **Central Panel**: Detailed AI information
   - Three tabs: Communication, Decisions, Performance
   - Shows data for the selected AI

### Adding AI Players

Click the "Add AI" button in the left panel to add a new AI player with default settings. You can also use the HTTP API:

```bash
curl -X POST http://localhost:14191/ai/add \
  -d '{"difficulty": 0.8, "personality": "aggressive"}'
```

## Debug Information

### AI States
- **Hat**: Current role (Pilot, Gunner, Engineer, etc.)
- **Action**: What the AI is currently doing
- **Confidence**: How certain the AI is about its decision (0-100%)
- **Health Status**: Current health level

### Communication Types
- **Commands**: Orders from captain to team
- **Status Updates**: AI reporting its state
- **Requests**: Asking for help or resources
- **Intel**: Sharing enemy/resource locations
- **Coordination**: Team movement/strategy

### Decision Types
- Movement decisions
- Station operation choices
- Combat actions
- Resource collection
- Team coordination

## Troubleshooting

### Connection Issues
- Ensure the game server is running on port 14191
- Check that the `/debug` WebSocket endpoint is available
- Look for connection errors in the console

### No AI Data
- Verify AI players exist (check left panel)
- Ensure AI system is running (not paused)
- Check server logs for AI-related errors

### Performance
- Reduce update frequency if experiencing lag
- Limit number of AI players for testing
- Check system resources

## Future Enhancements

### Planned Features
- Visual communication graph with node layout
- Heatmap of AI movement patterns
- Decision tree visualization
- Export/import of AI configurations
- Replay system for analyzing past games
- Machine learning integration hooks

### Debug Modes
- Limited vision simulation
- Stress testing with many AIs
- Scenario playback
- A/B testing different AI configurations

## Technical Details

The debug client connects to the server via WebSocket at `ws://localhost:14191/debug` and receives:
- Game state updates
- AI-specific visualization data
- Simulation control acknowledgments

It's built with:
- **egui**: Immediate mode GUI
- **eframe**: Native window management
- **ws**: WebSocket client
- **serde**: Message serialization