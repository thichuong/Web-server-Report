//! Layer 3: Communication
//!
//! This layer handles all communication-related functionality including:
//! - WebSocket real-time communication
//! - HTTP request/response handling
//! - Message broadcasting and routing
//! - Data communication with infrastructure layer
//! - Dashboard data communication and caching
//! - Layer 2 adapters for clean API access
//! - Layer 2 gRPC client for high-performance microservice communication

pub mod websocket_service;
pub mod data_communication;
pub mod dashboard_communication;
pub mod layer2_adapters;
pub mod layer2_grpc_client;

