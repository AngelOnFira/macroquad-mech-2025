// WebSocket bindings for macroquad WASM
// This provides the network functionality without using wasm-bindgen

const sockets = new Map();
let nextSocketId = 1;

// Message queue for each socket
const messageQueues = new Map();

// Register the module that will be imported by WASM
miniquad_add_plugin({
    register_plugin: function (importObject) {
        importObject.network_bindings = {
            js_ws_connect: function (urlPtr, urlLen) {
                const url = UTF8ToString(urlPtr, urlLen);
                const socketId = nextSocketId++;

                try {
                    const ws = new WebSocket(url);
                    ws.binaryType = 'arraybuffer';

                    // Initialize message queue
                    messageQueues.set(socketId, []);

                    ws.onopen = function () {
                        console.log(`WebSocket ${socketId} connected to ${url}`);
                    };

                    ws.onmessage = function (event) {
                        if (typeof event.data === 'string') {
                            messageQueues.get(socketId).push(event.data);
                        }
                    };

                    ws.onerror = function (error) {
                        console.error(`WebSocket ${socketId} error:`, error);
                    };

                    ws.onclose = function (event) {
                        console.log(`WebSocket ${socketId} closed: code=${event.code}, reason=${event.reason}`);
                        sockets.delete(socketId);
                        messageQueues.delete(socketId);
                    };

                    sockets.set(socketId, ws);
                    return socketId;
                } catch (e) {
                    console.error('Failed to create WebSocket:', e);
                    return 0;
                }
            },

            js_ws_send: function (socketId, dataPtr, dataLen) {
                const socket = sockets.get(socketId);
                if (socket && socket.readyState === WebSocket.OPEN) {
                    const data = UTF8ToString(dataPtr, dataLen);
                    socket.send(data);
                }
            },

            js_ws_close: function (socketId) {
                const socket = sockets.get(socketId);
                if (socket) {
                    socket.close();
                    sockets.delete(socketId);
                    messageQueues.delete(socketId);
                }
            },

            js_ws_is_connected: function (socketId) {
                const socket = sockets.get(socketId);
                return (socket && socket.readyState === WebSocket.OPEN) ? 1 : 0;
            },

            js_ws_poll_message: function (socketId, bufferPtr, bufferLen) {
                const queue = messageQueues.get(socketId);
                if (!queue || queue.length === 0) {
                    return -1; // No messages
                }

                const message = queue.shift();
                const bytes = new TextEncoder().encode(message);

                if (bytes.length > bufferLen) {
                    console.error('Message too large for buffer');
                    return -1;
                }

                // Copy message to WASM memory
                const buffer = new Uint8Array(wasm_memory.buffer, bufferPtr, bufferLen);
                buffer.set(bytes);

                return bytes.length;
            },
        };


        var new_env = {
            // Settings persistence functions
            js_settings_save: function (keyPtr, keyLen, valuePtr, valueLen) {
                try {
                    const key = UTF8ToString(keyPtr, keyLen);
                    const value = UTF8ToString(valuePtr, valueLen);
                    localStorage.setItem(`mech_battle_arena_${key}`, value);
                    return 1; // Success
                } catch (e) {
                    console.error('Failed to save settings:', e);
                    return 0; // Failure
                }
            },

            js_settings_load: function (keyPtr, keyLen, bufferPtr, bufferLen) {
                try {
                    const key = UTF8ToString(keyPtr, keyLen);
                    const value = localStorage.getItem(`mech_battle_arena_${key}`);

                    if (value === null) {
                        return -1; // Key not found
                    }

                    const bytes = new TextEncoder().encode(value);

                    if (bytes.length > bufferLen) {
                        console.error('Settings value too large for buffer');
                        return -1;
                    }

                    // Copy value to WASM memory
                    const buffer = new Uint8Array(wasm_memory.buffer, bufferPtr, bufferLen);
                    buffer.set(bytes);

                    return bytes.length;
                } catch (e) {
                    console.error('Failed to load settings:', e);
                    return -1;
                }
            },

            js_settings_exists: function (keyPtr, keyLen) {
                try {
                    const key = UTF8ToString(keyPtr, keyLen);
                    return localStorage.getItem(`mech_battle_arena_${key}`) !== null ? 1 : 0;
                } catch (e) {
                    console.error('Failed to check settings:', e);
                    return 0;
                }
            }
        };

        // Go through all the keys in the new_env object and add them to the importObject.env object
        for (const key in new_env) {
            importObject.env[key] = new_env[key];
        }
    }
});