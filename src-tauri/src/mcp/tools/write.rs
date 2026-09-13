//! The three write tools. Task 5 fills these in by TDD; for now they refuse.

use rusqlite::Connection;
use serde_json::Value;

use crate::error::{AppError, AppResult};
use crate::mcp::config::McpConfig;

pub fn categorize(_conn: &Connection, _cfg: &McpConfig, _args: &Value) -> AppResult<Value> {
    Err(AppError::Invalid("not implemented".into()))
}

pub fn create_rule(_conn: &mut Connection, _cfg: &McpConfig, _args: &Value) -> AppResult<Value> {
    Err(AppError::Invalid("not implemented".into()))
}

pub fn settle_bill(_conn: &Connection, _cfg: &McpConfig, _args: &Value) -> AppResult<Value> {
    Err(AppError::Invalid("not implemented".into()))
}
