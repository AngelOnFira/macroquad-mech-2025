#!/bin/bash

# Build script for macroquad WebAssembly client

echo "Building WebAssembly client with macroquad..."

# Clean previous builds
rm -rf web_build
mkdir -p web_build

# Build with cargo for wasm32 target - macroquad handles the wasm-bindgen internally
cd client
cargo build --target wasm32-unknown-unknown --no-default-features --features web

# Copy the wasm file with the expected name
cp ../target/wasm32-unknown-unknown/debug/client.wasm ../web_build/mech-battle-arena.wasm

# Create macroquad-compatible HTML
cat > ../web_build/index.html << 'EOF'
<!DOCTYPE html>
<html lang="en">
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
    <canvas id="glcanvas" tabindex='1'></canvas>
    <script src="https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js"></script>
    <script>load("mech-battle-arena.wasm");</script>
</body>
</html>
EOF

# Create a simple web server script
cat > ../web_build/serve.py << 'EOF'
#!/usr/bin/env python3
import http.server
import socketserver
import os

class MyHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        super().end_headers()

    def guess_type(self, path):
        mimetype = super().guess_type(path)
        if path.endswith('.wasm'):
            return 'application/wasm'
        return mimetype

PORT = 8080
os.chdir(os.path.dirname(os.path.abspath(__file__)))

with socketserver.TCPServer(("", PORT), MyHTTPRequestHandler) as httpd:
    print(f"Server running at http://localhost:{PORT}/")
    httpd.serve_forever()
EOF

chmod +x ../web_build/serve.py

echo "Build complete! Files are in web_build/"
echo "To run:"
echo "  1. Start the game server: cargo run --bin server"
echo "  2. In another terminal, go to web_build/ and run: ./serve.py"
echo "  3. Open http://localhost:8080 in your browser"