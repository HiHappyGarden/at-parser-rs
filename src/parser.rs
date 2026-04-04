/***************************************************************************
 *
 * AT Command Parser
 * Copyright (C) 2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, see <https://www.gnu.org/licenses/>.
 *
 ***************************************************************************/
 
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
///
/// Generic over `T` which must implement the [`AtContext<SIZE>`](crate::context::AtContext) trait,
/// and over the const `SIZE` which determines the response buffer size.
///
/// # Generic Design
///
/// The parser is generic over the command handler type `T` and response size `SIZE` to allow
/// compile-time type checking when all handlers are of the same type. This provides:
///
/// - **Type safety**: Compile-time verification of handler types
/// - **Zero overhead**: No dynamic dispatch when using concrete types
/// - **Flexibility**: Can be used with trait objects (`dyn AtContext<SIZE>`) for mixed handler types
///
/// # Usage Patterns
///
/// ## With trait objects (recommended for mixed types):
/// ```rust,no_run
/// # use at_parser_rs::parser::AtParser;
/// # use at_parser_rs::context::AtContext;
/// # struct Dummy; impl AtContext<64> for Dummy {}
/// # let mut echo_handler = Dummy; let mut reset_handler = Dummy;
/// const SIZE: usize = 64;
/// let mut parser: AtParser<dyn AtContext<SIZE>, SIZE> = AtParser::new();
/// let commands: &mut [(&str, &mut dyn AtContext<SIZE>)] = &mut [
///     ("AT+ECHO", &mut echo_handler),
///     ("AT+RST", &mut reset_handler),
/// ];
/// parser.set_commands(commands);
/// ```
///
/// ## With concrete types (for homogeneous handlers):
/// ```rust,no_run
/// # use at_parser_rs::parser::AtParser;
/// # use at_parser_rs::context::AtContext;
/// # struct MyHandler; impl AtContext<64> for MyHandler {}
/// # let mut handler1 = MyHandler; let mut handler2 = MyHandler;
/// const SIZE: usize = 64;
/// let mut parser: AtParser<MyHandler, SIZE> = AtParser::new();
/// let commands: &mut [(&str, &mut MyHandler)] = &mut [
///     ("AT+CMD1", &mut handler1),
///     ("AT+CMD2", &mut handler2),
/// ];
/// parser.set_commands(commands);
/// ```
pub struct AtParser<'a, T, const SIZE: usize>
where
    T: AtContext<SIZE> + ?Sized {
    /// Array of registered commands with their name and handler
    pub commands: &'a mut [(&'static str, &'a mut T)],
}

impl<'a, T, const SIZE: usize> AtParser<'a, T, SIZE>
where
    T: AtContext<SIZE> + ?Sized {

    /// Create a new empty parser
    pub const fn new() -> Self {
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
    /// * `Ok(Bytes<SIZE>)` - Success response from the command handler
    /// * `Err(AtError)` - Error if parsing fails or command is not found
    pub fn execute<'b>(&'b mut self, input: &'b str) -> AtResult<'b, SIZE> {
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
fn parse<'a>(input: &'a str) -> Result<(&'a str, AtForm<'a>), AtError<'a>> {
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