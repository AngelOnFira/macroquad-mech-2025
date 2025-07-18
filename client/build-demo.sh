#!/bin/bash
# Build script for the demo WASM binary

echo "Building Demo WASM..."

# Build the demo binary for WASM
cargo build --bin demo --target wasm32-unknown-unknown

# Copy to dist with a different name
cp ../target/wasm32-unknown-unknown/debug/demo.wasm ../dist/demo.wasm

# Make sure we have the necessary JS files
if [ ! -f ../dist/gl.js ] || [ ! -f ../dist/mq_js_bundle.js ]; then
    echo "Required JS files missing. Building main client first..."
    ./build-simple.sh
fi

# Create a demo HTML file
cat > ../dist/demo.html << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <title>Mech Battle Arena - Hybrid Tile Demo</title>
    <style>
        html, body {
            margin: 0;
            padding: 0;
            overflow: hidden;
            background-color: #0a0a0f;
            font-family: Arial, sans-serif;
            width: 100%;
            height: 100%;
        }
        canvas {
            position: absolute;
            width: 100%;
            height: 100%;
            z-index: 0;
        }
        #info {
            position: absolute;
            top: 10px;
            right: 10px;
            color: white;
            background: rgba(0,0,0,0.7);
            padding: 10px;
            border-radius: 5px;
            z-index: 1;
        }
    </style>
</head>
<body>
    <canvas id="glcanvas" tabindex='1'></canvas>
    <div id="info">
        <h3>Hybrid Tile System Demo</h3>
        <p>Arrow keys: Move camera</p>
        <p>V: Toggle vision/fog of war</p>
        <p>ESC: Exit demo</p>
    </div>
    <script src="https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js"></script>
    <script>
        load("./demo.wasm");
    </script>
</body>
</html>
EOF

echo "Demo build complete! Files in ../dist/"
echo "Access the demo at: http://localhost:8080/demo.html"