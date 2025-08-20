//! Layer 3: Communication
//! 
//! This layer handles all communication-related functionality including:
//! - WebSocket real-time communication
//! - HTTP request/response handling
//! - Message broadcasting and routing

pub mod websocket_service;

pub use websocket_service::WebSocketServiceIsland;
