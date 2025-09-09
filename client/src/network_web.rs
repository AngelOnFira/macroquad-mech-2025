// WebSocket implementation for browsers using macroquad's JS interop
// This avoids wasm-bindgen to stay compatible with macroquad's loader

#[cfg(target_arch = "wasm32")]
use crate::game_state::GameState;
#[cfg(target_arch = "wasm32")]
use crate::network_common::handle_server_message;
#[cfg(target_arch = "wasm32")]
use crate::network_trait::{NetworkClient as NetworkClientTrait, WebNetworkClient};
#[cfg(target_arch = "wasm32")]
use macroquad::prelude::*;
#[cfg(target_arch = "wasm32")]
use shared::*;
#[cfg(target_arch = "wasm32")]
use std::sync::{Arc, Mutex};

// JavaScript bindings for WebSocket using macroquad's sapp_jsutils
#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "network_bindings")]
extern "C" {
    fn js_ws_connect(url_ptr: *const u8, url_len: usize) -> u32;
    fn js_ws_send(socket_id: u32, data_ptr: *const u8, data_len: usize);
    fn js_ws_send_binary(socket_id: u32, data_ptr: *const u8, data_len: usize);
    fn js_ws_close(socket_id: u32);
    fn js_ws_is_connected(socket_id: u32) -> u32;
    fn js_ws_poll_message(socket_id: u32, buffer_ptr: *mut u8, buffer_len: usize) -> i32;
    fn js_ws_poll_binary_message(socket_id: u32, buffer_ptr: *mut u8, buffer_len: usize) -> i32;
}

#[cfg(target_arch = "wasm32")]
pub struct NetworkClient {
    socket_id: u32,
    game_state: Arc<Mutex<GameState>>,
    message_buffer: Vec<u8>,
}

#[cfg(target_arch = "wasm32")]
impl NetworkClientTrait for NetworkClient {
    type Error = String;

    fn connect(url: &str, game_state: Arc<Mutex<GameState>>) -> Result<Self, Self::Error> {
        let socket_id = unsafe { js_ws_connect(url.as_ptr(), url.len()) };

        if socket_id == 0 {
            return Err("Failed to create WebSocket".to_string());
        }

        info!("WebSocket connecting to: {}", url);

        Ok(NetworkClient {
            socket_id,
            game_state,
            message_buffer: vec![0u8; 65536], // 64KB buffer for messages
        })
    }

    fn send_message(&self, msg: ClientMessage) {
        if let Ok(bytes) = rmp_serde::to_vec(&msg) {
            unsafe {
                js_ws_send_binary(self.socket_id, bytes.as_ptr(), bytes.len());
            }
        }
    }

    fn is_connected(&self) -> bool {
        unsafe { js_ws_is_connected(self.socket_id) != 0 }
    }
}

#[cfg(target_arch = "wasm32")]
impl WebNetworkClient for NetworkClient {
    fn update(&mut self) {
        // Poll for binary messages first (primary MessagePack format)
        loop {
            let msg_len = unsafe {
                js_ws_poll_binary_message(
                    self.socket_id,
                    self.message_buffer.as_mut_ptr(),
                    self.message_buffer.len(),
                )
            };

            if msg_len < 0 {
                break; // No more binary messages
            }

            // Parse the binary message
            if let Ok(server_msg) = rmp_serde::from_slice::<ServerMessage>(&self.message_buffer[0..msg_len as usize]) {
                handle_server_message(server_msg, &self.game_state);
            } else {
                error!("Failed to parse binary server message, length: {}", msg_len);
            }
        }

        // Poll for legacy text messages (JSON)
        loop {
            let msg_len = unsafe {
                js_ws_poll_message(
                    self.socket_id,
                    self.message_buffer.as_mut_ptr(),
                    self.message_buffer.len(),
                )
            };

            if msg_len < 0 {
                break; // No more text messages
            }

            // Parse the text message
            if let Ok(message_str) = std::str::from_utf8(&self.message_buffer[0..msg_len as usize])
            {
                if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(message_str) {
                    handle_server_message(server_msg, &self.game_state);
                } else {
                    error!("Failed to parse JSON server message: {}", message_str);
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl Drop for NetworkClient {
    fn drop(&mut self) {
        unsafe {
            js_ws_close(self.socket_id);
        }
    }
}

