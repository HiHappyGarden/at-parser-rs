use crate::context::AtContext;
use crate::{AtError, AtResult, Args};

/*
AT Command Forms:
- AT+CMD     (execution)
- AT+CMD?    (query)
- AT+CMD=?   (test)
- AT+CMD=... (set with arguments)
 */

/// Represents the different forms an AT command can take
enum AtForm<'a> {
    /// Execute command without parameters (AT+CMD)
    Exec,
    /// Query the current state (AT+CMD?)
    Query,
    /// Test command availability or get valid ranges (AT+CMD=?)
    Test,
    /// Set command with arguments (AT+CMD=args)
    Set(Args<'a>),
}

/// The main AT command parser
/// Generic over T which must implement AtContext trait
pub struct AtParser<'a, T>
where
    T: AtContext {
    /// Array of registered commands with their name and handler
    pub commands: &'a mut [(&'static str, &'a mut T)],
}

impl<'a, T> AtParser<'a, T>
where
    T: AtContext {

    /// Create a new empty parser
    pub fn new() -> Self {
        Self { commands: & mut [] }
    }

    /// Register commands that this parser will handle
    pub fn set_commands(&mut self, commands: &'a mut [(&'static str, &'a mut T)]) {
        self.commands = commands;
    }

    /// Parse and execute an AT command string
    /// 
    /// # Arguments
    /// * `input` - The raw AT command string (e.g., "AT+CMD?")
    /// 
    /// # Returns
    /// * `Ok(&str)` - Success response from the command handler
    /// * `Err(AtError)` - Error if parsing fails or command is not found
    pub fn execute(&mut self, input: &str) -> AtResult<'static> {
        let input = input.trim();
        let (name, form) = parse(input)?;

        // Find the command handler
        let (_, module) = self.commands
            .iter_mut()
            .find(|(n, _)| *n == name)
            .ok_or(AtError::UnknownCommand)?;

        // Dispatch to the appropriate handler method
        match form {
            AtForm::Exec => module.exec(),
            AtForm::Query => module.query(),
            AtForm::Test => module.test(),
            AtForm::Set(args) => module.set(args),
        }
    }
}

/// Parse an AT command string into its name and form
/// 
/// # Arguments
/// * `input` - The command string to parse
/// 
/// # Returns
/// A tuple of (command_name, command_form)
fn parse<'a>(input: &'a str) -> Result<(&'a str, AtForm<'a>), AtError> {
    let input = input.trim();

    // Check suffixes to determine command form
    if let Some(cmd) = input.strip_suffix("=?") {
        Ok((cmd, AtForm::Test))
    } else if let Some(cmd) = input.strip_suffix('?') {
        Ok((cmd, AtForm::Query))
    } else if let Some((cmd, args)) = input.split_once('=') {
        Ok((cmd, AtForm::Set(Args { raw: args })))
    } else {
        Ok((input, AtForm::Exec))
    }
}