# DevContainer with Claude CLI and Playwright MCP

This devcontainer provides a complete environment for running Claude CLI with Playwright MCP in headless mode, perfect for testing on systems where Playwright MCP isn't natively supported.

## Features

- Ubuntu 22.04 base with all dependencies for headless Chrome
- Rust toolchain with WASM support
- Node.js 20.x
- Claude CLI pre-installed
- Playwright with headless Chrome
- Playwright MCP server configured
- Xvfb for headless display (DISPLAY=:99)
- All Mech Battle Arena development dependencies

## Quick Start

1. **Open in VS Code with DevContainers extension**:
   ```bash
   code .
   # Then: F1 -> "Dev Containers: Reopen in Container"
   ```

2. **Or use Docker directly**:
   ```bash
   # Build the container
   docker build -f .devcontainer/Dockerfile -t mech-arena-dev .
   
   # Run interactively with port forwarding
   docker run -it --rm \
     -v $(pwd):/workspace \
     -p 14191:14191 \
     -p 8080:8080 \
     mech-arena-dev
   ```

## Using Claude CLI with Playwright MCP

1. **Inside the container, first login to Claude**:
   ```bash
   claude login
   ```

2. **Run the test script to verify everything works**:
   ```bash
   /workspace/.devcontainer/test-playwright-mcp.sh
   ```

3. **Start a Claude session with Playwright MCP**:
   ```bash
   claude chat
   ```

   Example commands you can use:
   - "Navigate to http://localhost:8080 and take a screenshot"
   - "Click on the 'Join Game' button"
   - "Fill in the player name field with 'TestPlayer'"

## MCP Configuration

The Playwright MCP is configured in `~/.config/claude/config.json` with:
- Headless mode enabled
- Display set to :99 (Xvfb virtual display)

## Testing the Game

1. **Start the game servers**:
   ```bash
   cd /workspace
   just dev
   ```

2. **In another terminal, use Claude to test**:
   ```bash
   claude chat
   # Then ask: "Navigate to http://localhost:8080 and verify the game loads"
   ```

## Troubleshooting

- **Xvfb not running**: The container automatically starts Xvfb, but you can manually start it:
  ```bash
  Xvfb :99 -screen 0 1920x1080x24 -nolisten tcp &
  export DISPLAY=:99
  ```

- **Playwright issues**: Make sure to use the `--no-sandbox` flag in headless mode:
  ```javascript
  const browser = await chromium.launch({ 
    headless: true,
    args: ['--no-sandbox', '--disable-setuid-sandbox']
  });
  ```

- **Claude CLI not finding MCP**: Ensure the config is in the right place:
  ```bash
  cat ~/.config/claude/config.json
  ```

## Environment Variables

- `DISPLAY=:99` - Virtual display for headless operation
- `HEADLESS=true` - Forces Playwright to run in headless mode
- `RUST_LOG=info` - Rust logging level

## Notes

- The container includes all dependencies for the Mech Battle Arena project
- Ports 14191 (game server) and 8080 (web server) are exposed
- The workspace is mounted at `/workspace`
- All Playwright browser binaries are pre-installed