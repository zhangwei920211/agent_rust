//! MCP server 工具面（M1 Task5）：rmcp stdio，#[tool_router(server_handler)]。
//! 本任务占位。错误分层：业务错误走工具级 Ok(CallToolResult::error)（用户可见），
//! 仅协议异常用 Err(McpError)（EXT-4）。工具无状态，跨调用状态用显式 handle（EXT-1）。
