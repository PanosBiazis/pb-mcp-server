//! Tool registry and catalog for pb-mcp-server
//!
//! Provides a centralized registry of all available MCP tools with
//! metadata including name, description, profile, risk level, and
//! category. Used by the profile system and MCP tool registration.

pub mod catalog;
pub mod registry;

pub use catalog::ToolCatalog;
pub use registry::ToolRegistry;
