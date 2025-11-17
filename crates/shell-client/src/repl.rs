//! Interactive REPL (Read-Eval-Print-Loop)

use crate::{client::Client, ClientError, Result};
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use shell_proto::CommandStatus;
use std::sync::Arc;
use tracing::{debug, error};

/// Interactive REPL
pub struct Repl {
    /// Client connection
    client: Arc<Client>,

    /// Readline editor
    editor: DefaultEditor,
}

impl Repl {
    /// Create a new REPL
    pub fn new(client: Client) -> Self {
        let editor = DefaultEditor::new().expect("Failed to create readline editor");

        Self {
            client: Arc::new(client),
            editor,
        }
    }

    /// Run the REPL
    pub async fn run(&mut self) -> Result<()> {
        println!("{}", "Reticulum Shell Client".bold().green());
        println!("Type 'help' for commands, 'exit' to quit\n");

        loop {
            let prompt = "rsh> ".cyan().to_string();

            match self.editor.readline(&prompt) {
                Ok(line) => {
                    let line = line.trim();

                    // Skip empty lines
                    if line.is_empty() {
                        continue;
                    }

                    // Add to history
                    let _ = self.editor.add_history_entry(line);

                    // Handle special commands
                    if let Some(result) = self.handle_special_command(line).await? {
                        if !result {
                            break; // Exit requested
                        }
                        continue;
                    }

                    // Parse and execute command
                    match self.execute_line(line).await {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("{} {}", "Error:".red().bold(), e);
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    println!("^D");
                    break;
                }
                Err(err) => {
                    error!("Readline error: {:?}", err);
                    return Err(ClientError::Repl(err.to_string()));
                }
            }
        }

        // Disconnect before exiting
        self.client.disconnect().await?;

        Ok(())
    }

    /// Handle special built-in commands
    async fn handle_special_command(&self, line: &str) -> Result<Option<bool>> {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            return Ok(None);
        }

        match parts[0] {
            "exit" | "quit" => {
                println!("Goodbye!");
                return Ok(Some(false));
            }
            "help" => {
                self.print_help();
                return Ok(Some(true));
            }
            "status" => {
                self.print_status().await;
                return Ok(Some(true));
            }
            "clear" => {
                print!("\x1B[2J\x1B[1;1H"); // ANSI clear screen
                return Ok(Some(true));
            }
            _ => {}
        }

        Ok(None)
    }

    /// Execute a command line
    async fn execute_line(&self, line: &str) -> Result<()> {
        // Parse command line
        let parts = shell_words::split(line)
            .map_err(|e| ClientError::Repl(format!("Invalid command syntax: {}", e)))?;

        if parts.is_empty() {
            return Ok(());
        }

        let command = parts[0].clone();
        let args = parts[1..].to_vec();

        debug!(command = %command, args = ?args, "Executing command");

        // Execute command
        let response = self.client.execute_command(command, args).await?;

        // Display output
        match response.status {
            CommandStatus::Success => {
                // Print stdout
                if !response.stdout.is_empty() {
                    print!("{}", String::from_utf8_lossy(&response.stdout));
                }

                // Print stderr in red
                if !response.stderr.is_empty() {
                    eprint!("{}", String::from_utf8_lossy(&response.stderr).red());
                }
            }
            CommandStatus::Error => {
                eprintln!(
                    "{} Exit code: {}",
                    "Command failed:".red().bold(),
                    response.exit_code
                );
                if !response.stderr.is_empty() {
                    eprint!("{}", String::from_utf8_lossy(&response.stderr).red());
                }
            }
            CommandStatus::Timeout => {
                eprintln!("{}", "Command timed out".red().bold());
            }
            CommandStatus::Killed => {
                eprintln!("{}", "Command was killed".red().bold());
            }
        }

        Ok(())
    }

    /// Print help message
    fn print_help(&self) {
        println!("{}", "Available commands:".bold());
        println!("  help          - Show this help message");
        println!("  status        - Show connection status");
        println!("  clear         - Clear screen");
        println!("  exit, quit    - Exit the shell");
        println!("\nAny other command will be executed on the remote server.");
    }

    /// Print connection status
    async fn print_status(&self) {
        let connected = self.client.is_connected().await;

        println!("{}", "Connection Status:".bold());
        if connected {
            println!("  Status: {}", "Connected".green().bold());
        } else {
            println!("  Status: {}", "Disconnected".red().bold());
        }
    }
}
