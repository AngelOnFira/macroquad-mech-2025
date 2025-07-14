#!/bin/bash

# Build with wasm-bindgen instead of macroquad's loader

echo "Building with wasm-bindgen..."

# Install wasm-bindgen-cli if not present
if ! command -v wasm-bindgen &> /dev/null; then
    echo "Installing wasm-bindgen-cli..."
    cargo install wasm-bindgen-cli
fi

# Clean and create build directory
rm -rf web_build
mkdir -p web_build

# Build the WASM module
cd client
cargo build --target wasm32-unknown-unknown --no-default-features --features web
cd ..

# Generate bindings with wasm-bindgen
wasm-bindgen target/wasm32-unknown-unknown/debug/client.wasm \
    --out-dir web_build \
    --target web \
    --no-typescript

# Create index.html that loads wasm-bindgen properly
cat > web_build/index.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Mech Battle Arena</title>
    <style>
        html, body {
            margin: 0;
            padding: 0;
            width: 100%;
            height: 100%;
            overflow: hidden;
            background-color: black;
        }
        canvas {
            display: block;
            width: 100%;
            height: 100%;
        }
        #loading {
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            color: white;
            font-family: Arial, sans-serif;
            font-size: 24px;
        }
    </style>
</head>
<body>
    <div id="loading">Loading game...</div>
    <canvas id="glcanvas" tabindex='1'></canvas>
    
    <script type="module">
        import init from './client.js';
        
        async function run() {
            await init();
            document.getElementById("loading").style.display = "none";
        }
        
        run();
    </script>
</body>
</html>
EOF

# Create the Python server
cat > web_build/serve.py << 'EOF'
#!/usr/bin/env python3
import http.server
import socketserver
import os

class MyHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        # Required headers for SharedArrayBuffer
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        # CORS headers
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type')
        super().end_headers()

    def guess_type(self, path):
        mimetype = super().guess_type(path)
        if path.endswith('.wasm'):
            return 'application/wasm'
        elif path.endswith('.js'):
            return 'application/javascript'
        return mimetype

PORT = 8080
os.chdir(os.path.dirname(os.path.abspath(__file__)))

with socketserver.TCPServer(("", PORT), MyHTTPRequestHandler) as httpd:
    print(f"Server running at http://localhost:{PORT}/")
    httpd.serve_forever()
EOF

chmod +x web_build/serve.py

echo "Build complete!"
echo "To run:"
echo "  1. Start game server: cargo run --bin server"
echo "  2. Start web server: cd web_build && ./serve.py"
echo "  3. Open http://localhost:8080"