use jsonrpc_v2::Error as RpcError;
use mcp_protocol_sdk::McpError;

pub trait ToRpcError {
    fn to_rpc_error(self) -> RpcError;
}

impl ToRpcError for McpError {
    fn to_rpc_error(self) -> RpcError {
        match self {
            McpError::Validation(msg) => RpcError::Full {
                code: -32602,
                message: "Validation error".to_string(),
                data: Some(Box::new(msg)),
            },
            McpError::ToolNotFound(msg) => RpcError::Full {
                code: -32601,
                message: "Tool not found".to_string(),
                data: Some(Box::new(msg)),
            },
            McpError::ResourceNotFound(msg) => RpcError::Full {
                code: -32004,
                message: "Resource not found".to_string(),
                data: Some(Box::new(msg)),
            },
            McpError::PromptNotFound(msg) => RpcError::Full {
                code: -32005,
                message: "Prompt not found".to_string(),
                data: Some(Box::new(msg)),
            },
            McpError::Timeout(msg) => RpcError::Full {
                code: -32000,
                message: "Timeout".to_string(),
                data: Some(Box::new(msg)),
            },
            McpError::Cancelled(msg) => RpcError::Full {
                code: -32002,
                message: "Operation cancelled".to_string(),
                data: Some(Box::new(msg)),
            },
            McpError::Authentication(msg) => RpcError::Full {
                code: -32003,
                message: "Authentication error".to_string(),
                data: Some(Box::new(msg)),
            },
            McpError::Internal(msg) => RpcError::Full {
                code: -32603,
                message: "Internal error".to_string(),
                data: Some(Box::new(msg)),
            },
            McpError::Transport(msg)
            | McpError::Protocol(msg)
            | McpError::Serialization(msg)
            | McpError::InvalidUri(msg)
            | McpError::Connection(msg)
            | McpError::Io(msg)
            | McpError::Url(msg) => RpcError::Full {
                code: -32603,
                message: "Internal error".to_string(),
                data: Some(Box::new(msg)),
            },
        }
    }
}
