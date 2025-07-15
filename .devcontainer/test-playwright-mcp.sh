#!/bin/bash

echo "=== Testing Playwright MCP in Headless Mode ==="
echo

# Start Xvfb if not already running
if ! pgrep -x "Xvfb" > /dev/null; then
    echo "Starting Xvfb..."
    Xvfb :99 -screen 0 1920x1080x24 -nolisten tcp &
    sleep 2
fi

export DISPLAY=:99

# Test 1: Verify Playwright installation
echo "1. Testing Playwright installation..."
npx playwright --version
if [ $? -eq 0 ]; then
    echo "✓ Playwright is installed"
else
    echo "✗ Playwright is not installed properly"
    exit 1
fi
echo

# Test 2: Run a simple Playwright test
echo "2. Running a simple headless browser test..."
cat > /tmp/test-playwright.js << 'EOF'
const { chromium } = require('playwright');

(async () => {
    console.log('Launching headless browser...');
    const browser = await chromium.launch({ 
        headless: true,
        args: ['--no-sandbox', '--disable-setuid-sandbox']
    });
    
    const page = await browser.newPage();
    console.log('Navigating to example.com...');
    await page.goto('https://example.com');
    
    const title = await page.title();
    console.log(`Page title: ${title}`);
    
    await browser.close();
    console.log('Browser closed successfully');
})();
EOF

node /tmp/test-playwright.js
if [ $? -eq 0 ]; then
    echo "✓ Playwright headless test passed"
else
    echo "✗ Playwright headless test failed"
    exit 1
fi
echo

# Test 3: Test MCP server
echo "3. Testing Playwright MCP server..."
echo "Creating test configuration..."

# Create a temporary test to verify MCP works
cat > /tmp/test-mcp.js << 'EOF'
const { spawn } = require('child_process');

console.log('Starting Playwright MCP server...');
const mcp = spawn('npx', ['-y', '@modelcontextprotocol/server-playwright'], {
    env: { ...process.env, DISPLAY: ':99', HEADLESS: 'true' }
});

mcp.stdout.on('data', (data) => {
    console.log(`MCP stdout: ${data}`);
});

mcp.stderr.on('data', (data) => {
    console.log(`MCP stderr: ${data}`);
});

setTimeout(() => {
    console.log('Stopping MCP server...');
    mcp.kill();
    process.exit(0);
}, 5000);
EOF

node /tmp/test-mcp.js
echo "✓ MCP server test completed"
echo

# Test 4: Setup Claude CLI configuration
echo "4. Setting up Claude CLI configuration..."
mkdir -p ~/.config/claude

# Copy the MCP config
cp /workspace/.devcontainer/claude-mcp-config.json ~/.config/claude/config.json
echo "✓ Claude MCP configuration copied"
echo

echo "=== All tests completed ==="
echo
echo "To use Claude CLI with Playwright MCP:"
echo "1. First run: claude login"
echo "2. Then use: claude chat"
echo "3. The Playwright MCP will be available for browser automation"
echo
echo "The MCP configuration has been set up at: ~/.config/claude/config.json"
echo "Playwright will run in headless mode using Xvfb display :99"