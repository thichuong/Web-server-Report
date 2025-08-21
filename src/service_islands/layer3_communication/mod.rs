//! Layer 3: Communication
//! 
//! This layer handles all communication-related functionality including:
//! - WebSocket real-time communication
//! - HTTP request/response handling
//! - Message broadcasting and routing
//! - Data communication with infrastructure layer
//! - Layer 2 adapters for clean API access

pub mod websocket_service;
pub mod data_communication;
pub mod layer2_adapters;

