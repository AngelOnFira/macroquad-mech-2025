#!/bin/bash

# Build script for WebAssembly client

echo "Building WebAssembly client..."

# Clean previous builds
rm -rf web_build
mkdir -p web_build

# Build with cargo for wasm32 target
cd client
cargo build --target wasm32-unknown-unknown --release --no-default-features --features web

# Copy the wasm file
cp ../target/wasm32-unknown-unknown/release/client.wasm ../web_build/

# Download miniquad JS bundle if not present
if [ ! -f ../web_build/mq_js_bundle.js ]; then
    echo "Downloading miniquad JS bundle..."
    curl -o ../web_build/mq_js_bundle.js https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js
fi

# Create index.html
cat > ../web_build/index.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Mech Battle Arena</title>
    <style>
        body {
            margin: 0;
            padding: 0;
            background-color: #000;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
        }
        canvas {
            display: block;
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
    
    <script src="mq_js_bundle.js"></script>
    <script>
        load("client.wasm").then(() => {
            document.getElementById("loading").style.display = "none";
        });
    </script>
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
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type')
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