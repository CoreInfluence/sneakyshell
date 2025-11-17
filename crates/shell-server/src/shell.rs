//! Command execution functionality

use crate::{Result, ServerError};
use shell_proto::{CommandRequest, CommandResponse, CommandStatus};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::process::Command as TokioCommand;
use tokio::time::timeout;
use tracing::{debug, warn};

/// Command executor
pub struct CommandExecutor {
    /// Default timeout (seconds)
    default_timeout: u64,
}

impl CommandExecutor {
    /// Create a new command executor
    pub fn new(default_timeout: u64) -> Self {
        Self { default_timeout }
    }

    /// Execute a command
    pub async fn execute(&self, request: CommandRequest) -> Result<CommandResponse> {
        let start_time = Instant::now();

        debug!(
            id = request.id,
            command = %request.command,
            args = ?request.args,
            "Executing command"
        );

        // Determine timeout
        let cmd_timeout = Duration::from_secs(
            request.timeout.unwrap_or(self.default_timeout)
        );

        // Build command
        let mut cmd = TokioCommand::new(&request.command);
        cmd.args(&request.args);
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Set environment variables
        cmd.env_clear(); // Start with clean environment for security
        if let Some(env) = &request.env {
            for (key, value) in env {
                cmd.env(key, value);
            }
        }

        // Set working directory
        if let Some(work_dir) = &request.working_dir {
            cmd.current_dir(work_dir);
        }

        // Execute with timeout
        let result = timeout(cmd_timeout, cmd.output()).await;

        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(Ok(output)) => {
                let status = if output.status.success() {
                    CommandStatus::Success
                } else {
                    CommandStatus::Error
                };

                let exit_code = output.status.code().unwrap_or(-1);

                debug!(
                    id = request.id,
                    exit_code = exit_code,
                    stdout_len = output.stdout.len(),
                    stderr_len = output.stderr.len(),
                    duration_ms = execution_time_ms,
                    "Command completed"
                );

                Ok(CommandResponse {
                    id: request.id,
                    status,
                    stdout: output.stdout,
                    stderr: output.stderr,
                    exit_code,
                    execution_time_ms,
                })
            }
            Ok(Err(e)) => {
                warn!(id = request.id, error = %e, "Command execution failed");
                Ok(CommandResponse {
                    id: request.id,
                    status: CommandStatus::Error,
                    stdout: vec![],
                    stderr: format!("Execution error: {}", e).into_bytes(),
                    exit_code: -1,
                    execution_time_ms,
                })
            }
            Err(_) => {
                warn!(id = request.id, "Command timed out");
                Ok(CommandResponse {
                    id: request.id,
                    status: CommandStatus::Timeout,
                    stdout: vec![],
                    stderr: b"Command execution timed out".to_vec(),
                    exit_code: -1,
                    execution_time_ms,
                })
            }
        }
    }

    /// Validate a command request (security checks)
    pub fn validate_request(&self, request: &CommandRequest) -> Result<()> {
        // Check for empty command
        if request.command.is_empty() {
            return Err(ServerError::Execution("Command cannot be empty".to_string()));
        }

        // Prevent path traversal in working directory
        if let Some(work_dir) = &request.working_dir {
            if work_dir.contains("..") {
                return Err(ServerError::Execution(
                    "Path traversal not allowed in working directory".to_string(),
                ));
            }
        }

        // Additional security checks could be added here:
        // - Blacklist certain commands
        // - Validate arguments
        // - Check resource limits
        // etc.

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_simple_command() {
        let executor = CommandExecutor::new(30);
        let request = CommandRequest {
            id: 1,
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
            env: None,
            timeout: None,
            working_dir: None,
        };

        let response = executor.execute(request).await.unwrap();
        assert_eq!(response.status, CommandStatus::Success);
        assert_eq!(response.exit_code, 0);
        assert_eq!(String::from_utf8_lossy(&response.stdout).trim(), "hello");
    }

    #[tokio::test]
    async fn test_command_with_error() {
        let executor = CommandExecutor::new(30);
        let request = CommandRequest {
            id: 2,
            command: "ls".to_string(),
            args: vec!["/nonexistent".to_string()],
            env: None,
            timeout: None,
            working_dir: None,
        };

        let response = executor.execute(request).await.unwrap();
        assert_eq!(response.status, CommandStatus::Error);
        assert_ne!(response.exit_code, 0);
    }

    #[test]
    fn test_validate_request() {
        let executor = CommandExecutor::new(30);

        // Valid request
        let valid = CommandRequest {
            id: 1,
            command: "ls".to_string(),
            args: vec![],
            env: None,
            timeout: None,
            working_dir: Some("/tmp".to_string()),
        };
        assert!(executor.validate_request(&valid).is_ok());

        // Invalid: empty command
        let invalid_empty = CommandRequest {
            id: 2,
            command: "".to_string(),
            args: vec![],
            env: None,
            timeout: None,
            working_dir: None,
        };
        assert!(executor.validate_request(&invalid_empty).is_err());

        // Invalid: path traversal
        let invalid_traversal = CommandRequest {
            id: 3,
            command: "ls".to_string(),
            args: vec![],
            env: None,
            timeout: None,
            working_dir: Some("../../etc".to_string()),
        };
        assert!(executor.validate_request(&invalid_traversal).is_err());
    }
}
