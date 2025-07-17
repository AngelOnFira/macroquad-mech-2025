# Macroquad + wasm-bindgen Compatibility

## The Problem

Macroquad has its own JavaScript loader that's incompatible with standard wasm-bindgen output. The generated JavaScript from wasm-bindgen expects certain imports and structures that conflict with macroquad's loading mechanism.

## The Solution

We apply four patches to the wasm-bindgen generated JavaScript:

1. **Remove env import**: `import * as __wbg_star0 from 'env';` → (removed)
2. **Export set_wasm function**: `let wasm;` → `let wasm; export const set_wasm = (w) => wasm = w;`
3. **Fix imports**: `imports['env'] = __wbg_star0;` → `return imports.wbg;`
4. **Fix get_imports**: `const imports = __wbg_get_imports();` → `return __wbg_get_imports();`

## Implementation

### Current Approach (Rust-based)

1. **Trunk Hook**: `Trunk.toml` defines a post-build hook
2. **Rust Patcher**: `src/bin/patch_wasm.rs` applies the patches
3. **Custom HTML**: `index.html` uses macroquad's loading mechanism

```toml
# Trunk.toml
[[hooks]]
stage = "post_build"
command = "cargo"
command_arguments = ["run", "--bin", "patch_wasm"]
```

### How It Works

1. Trunk builds the WASM using wasm-bindgen
2. Post-build hook runs our Rust patcher
3. Patcher reads `dist/client.js` and applies the four patches
4. The patched JS works with macroquad's custom loader

## Why Not Pure Rust Build Integration?

Several challenges prevent a cleaner integration:

1. **Build Order**: wasm-bindgen runs *after* cargo build, so build.rs can't help
2. **Trunk Integration**: Trunk controls the wasm-bindgen invocation
3. **Text Manipulation**: The patches are string replacements on generated JS

Potential future solutions:
- Custom wasm-bindgen wrapper
- Trunk plugin system (if it gets one)
- Upstream fix in macroquad or wasm-bindgen

## Testing

Run `just dev` or `just build-web` and verify:
1. The build completes without errors
2. `dist/client.js` contains `export const set_wasm`
3. The game loads in the browser

## References

- [Original macroquad issue](https://github.com/not-fl3/macroquad/issues/212#issuecomment-835276147)
- [Macroquad JS bundle](https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js)