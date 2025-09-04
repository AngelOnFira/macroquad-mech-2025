---
name: menu-interface
status: backlog
created: 2025-09-03T18:41:42Z
updated: 2025-09-04T12:56:10Z
progress: 0%
prd: .claude/prds/menu-interface.md
github: https://github.com/AngelOnFira/macroquad-mech-2025/issues/2
---

# Epic: Menu Interface

## Overview

Implement a terminal-style keyboard-driven menu system by extending the existing egui integration and GameState management. The system will add main menu, settings, and pause functionality while leveraging the project's existing input handling patterns and UI infrastructure. Focus on minimal code addition by reusing existing abstractions.

## Architecture Decisions

### Leverage Existing Infrastructure
- **Use egui-macroquad**: Project already integrates egui for debug overlays - extend this for menus
- **Extend GameState**: Add menu states to existing state management rather than creating new system  
- **Reuse InputHandler**: Follow established input patterns for consistent keyboard navigation
- **Terminal Styling**: Use egui's customization to create terminal aesthetic within existing framework

### State Machine Design
```rust
enum AppState {
    MainMenu,
    Settings(SettingsScreen),
    InGame(GameState),
    Paused,
    Connecting,
}
```

### UI Framework Choice
- **egui for complex menus**: Settings screens with hierarchical navigation
- **Custom terminal styling**: Override egui theme for retro-futuristic appearance
- **Keyboard-first interaction**: Disable mouse, use WASD + Enter navigation patterns

## Technical Approach

### Frontend Components

#### State Management Extension
- Extend existing state pattern to include menu states
- Preserve existing GameState for in-game functionality  
- Add state transition logic for menu navigation

#### Menu Components (egui-based)
- **MainMenuScreen**: Terminal-style main interface using egui panels
- **SettingsScreen**: Hierarchical settings with egui trees and controls
- **PauseOverlay**: In-game pause menu as egui modal
- **TerminalTheme**: Custom egui styling for retro appearance

#### Input Integration
- Extend existing InputHandler for menu-specific keyboard navigation
- Map WASD to egui focus navigation
- Preserve game input patterns when transitioning between states

### Backend Services

#### Settings Management
- Browser localStorage for settings persistence
- Settings struct with serialization (already using serde patterns)
- Runtime settings application (resolution, audio levels when implemented)

#### State Persistence
- Save/restore last menu state
- Remember settings across sessions
- Handle connection state transitions cleanly

### Infrastructure

#### Integration Points
- Menu system integrates with existing network connection flow
- Settings apply to existing game systems (graphics, audio)
- Pause functionality works with current game loop structure

#### Minimal Dependencies
- No new major dependencies required
- Use existing egui-macroquad integration
- Leverage existing serde patterns for settings

## Implementation Strategy

### Phase 1: Core Menu Framework (3-4 days)
1. **Extend AppState**: Add menu states to existing state management
2. **Basic Main Menu**: Implement using egui with terminal styling
3. **State Transitions**: Connect menu to existing game start flow
4. **Input Mapping**: Extend InputHandler for keyboard menu navigation

### Phase 2: Settings System (2-3 days)  
1. **Settings Structure**: Create settings data model with serde
2. **Settings UI**: Hierarchical egui interface with categories
3. **Resolution Control**: Implement browser canvas resizing
4. **Placeholder Settings**: Mark unimplemented features clearly

### Phase 3: Integration & Polish (2-3 days)
1. **Pause Menu**: In-game egui overlay with menu options
2. **Terminal Styling**: Custom egui theme for retro appearance
3. **Settings Persistence**: localStorage integration
4. **Testing & Iteration**: Friend testing and refinement

## Task Breakdown Preview

High-level task categories (targeting <10 total tasks):
- [ ] **App State Extension**: Extend existing state machine for menu support
- [ ] **Main Menu Implementation**: Terminal-style main menu using egui
- [ ] **Settings Framework**: Data models, persistence, and basic UI structure  
- [ ] **Settings UI Implementation**: Graphics, audio, controls, network screens
- [ ] **Keyboard Navigation**: WASD menu navigation integration
- [ ] **Terminal Theme**: Custom egui styling for retro appearance
- [ ] **Pause Menu Integration**: In-game menu overlay system
- [ ] **State Transition Logic**: Menu to game flow integration
- [ ] **Settings Persistence**: Browser localStorage implementation
- [ ] **Testing & Polish**: Friend testing iteration and bug fixes

## Dependencies

### External Dependencies
- **egui-macroquad**: Already integrated, extend existing usage
- **Browser APIs**: localStorage for settings persistence
- **Existing input patterns**: WASD navigation consistency

### Internal Dependencies  
- **GameState architecture**: Extend without breaking existing functionality
- **InputHandler patterns**: Follow established keyboard input handling
- **Network connection flow**: Integrate menu system with existing multiplayer

### Prerequisite Work
- None - can build directly on existing infrastructure

## Success Criteria (Technical)

### Performance Benchmarks
- **Menu responsiveness**: <16ms input response for 60fps feel
- **State transition speed**: <100ms between menu screens
- **Memory usage**: <5MB additional for menu system
- **Rendering performance**: No impact on game loop performance

### Quality Gates
- **Keyboard-only navigation**: Complete menu system usable without mouse
- **Settings persistence**: Browser refresh preserves user preferences
- **State integrity**: Clean transitions without losing game state
- **Code reuse**: >70% of menu system built on existing infrastructure

### Acceptance Criteria
- Friend testers can navigate menus without instruction within 30 seconds
- Settings changes apply immediately with visual feedback
- Pause/resume functionality works seamlessly during gameplay
- Menu system serves as foundation for future UI needs (inventory, stations)

## Estimated Effort

### Overall Timeline: 7-10 days
- **Development**: 6-8 days (assuming part-time development)
- **Testing & iteration**: 1-2 days for friend feedback and refinement
- **Risk buffer**: Built into estimate for integration challenges

### Resource Requirements
- **Single developer**: Primary implementer familiar with existing codebase
- **Friend testers**: 3-5 people for usability validation
- **Design input**: Minimal - terminal aesthetic is well-defined

### Critical Path Items
1. **State management extension** - Foundation for all menu functionality
2. **egui terminal styling** - Must achieve desired aesthetic early
3. **Keyboard navigation** - Core interaction pattern must feel natural
4. **Settings persistence** - Required for user preference retention

## Risk Mitigation

### Technical Risks
- **egui styling limitations**: May require custom widgets for terminal appearance
- **Keyboard navigation in egui**: Framework primarily mouse-focused
- **State management complexity**: Adding menu states to existing game loop

### Mitigation Strategies
- **Rapid prototyping**: Test egui terminal styling early in Phase 1
- **Incremental integration**: Add menu states one at a time
- **Fallback options**: Simple text-based menus if egui styling proves difficult
- **Existing patterns**: Follow established input and state management approaches

## Tasks Created
- [ ] #3 - App State Extension (parallel: false, foundation task)
- [ ] #4 - Settings Data Model (parallel: true, depends on 3)
- [ ] #5 - Main Menu Implementation (parallel: true, depends on 3)
- [ ] #6 - Settings UI Implementation (parallel: false, depends on 4,5)
- [ ] #7 - Keyboard Navigation System (parallel: true, depends on 5)
- [ ] #8 - Terminal Theme Implementation (parallel: true, depends on 5)
- [ ] #9 - Pause Menu Integration (parallel: true, depends on 3,5)
- [ ] #10 - Settings Persistence (parallel: true, depends on 4)
- [ ] #11 - State Transition Logic (parallel: true, depends on 3, conflicts with 9)
- [ ] #12 - Testing & Polish (parallel: false, depends on 6,7,8,9)

Total tasks: 10
Parallel tasks: 7
Sequential tasks: 3
Estimated total effort: 41-50 hours (5-7 working days)