#!/bin/bash
set -e

# Determine if release build
PROFILE="debug"
if [[ "$1" == "--release" ]]; then
    PROFILE="release"
    CARGO_FLAGS="--release"
fi

echo "Building WASM for profile: $PROFILE"

# First, ensure wasm-bindgen-cli is installed
if ! command -v wasm-bindgen &> /dev/null; then
    echo "Installing wasm-bindgen-cli..."
    cargo install wasm-bindgen-cli
fi

# Build the WASM
echo "Building client for wasm32-unknown-unknown..."
cargo build --target wasm32-unknown-unknown --no-default-features $CARGO_FLAGS

# Create dist directory
mkdir -p ../dist

# Run wasm-bindgen
echo "Running wasm-bindgen..."
wasm-bindgen \
    ../target/wasm32-unknown-unknown/$PROFILE/client.wasm \
    --out-dir ../dist \
    --target web \
    --no-typescript

# Apply patches to make it work with macroquad
echo "Applying macroquad compatibility patches..."
sed -i '' "s/import \* as __wbg_star0 from 'env';//" ../dist/client.js
sed -i '' "s/let wasm;/let wasm; export const set_wasm = (w) => wasm = w;/" ../dist/client.js
sed -i '' "s/imports\['env'\] = __wbg_star0;/return imports.wbg;/" ../dist/client.js
sed -i '' "s/const imports = __wbg_get_imports();/return __wbg_get_imports();/" ../dist/client.js

# Create index.html
echo "Creating index.html..."
cat > ../dist/index.html << 'EOF'
<html lang="en">
<head>
    <meta charset="utf-8">
    <title>Mech Battle Arena</title>
    <style>
        html,
        body,
        canvas {
            margin: 0px;
            padding: 0px;
            width: 100%;
            height: 100%;
            overflow: hidden;
            position: absolute;
            z-index: 0;
        }
    </style>
</head>
<body style="margin: 0; padding: 0; height: 100vh; width: 100vw;">
    <canvas id="glcanvas" tabindex='1'></canvas>
    <script src="https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js"></script>
    <script type="module">
        import init, { set_wasm } from "./client.js";
        async function run() {
            let wbg = await init();
            miniquad_add_plugin({
                register_plugin: (a) => (a.wbg = wbg),
                on_init: () => set_wasm(wasm_exports),
                version: "0.0.1",
                name: "wbg",
            });
            load("./client_bg.wasm");
        }
        // Auto-start the game
        run();
    </script>
</body>
</html>
EOF

echo "Build complete! Files in ../dist/"