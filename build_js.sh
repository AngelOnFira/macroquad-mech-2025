#!/bin/bash

function wrap_js {
    echo "(function () {" >> dist/mq_js_bundle.js
    cat $1 >> dist/mq_js_bundle.js
    echo "}());" >> dist/mq_js_bundle.js
}
cat client/miniquad-gl.js > dist/mq_js_bundle.js
# wrap_js ../quad-snd/js/audio.js
# wrap_js ../sapp-jsutils/js/sapp_jsutils.js
# wrap_js ../quad-net/js/quad-net.js
# wrap_js ../tracing-wasm/tracing_wasm_plugin.js
wrap_js client/network_bindings.js
# cp ../tracing-wasm/tracing_wasm_plugin.js dist/mq_wasm_bindgen_bundle.js
