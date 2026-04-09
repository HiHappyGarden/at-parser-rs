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
    /// Array of registered commands with their command, AT response prefix, and handler
    pub commands: &'a mut [(&'static str, &'static str, &'a mut T)],
}

impl<'a, T, const SIZE: usize> AtParser<'a, T, SIZE>
where
    T: AtContext<SIZE> + ?Sized {

    /// Create a new empty parser with no registered commands.
    ///
    /// Call [`set_commands`](AtParser::set_commands) before dispatching any
    /// input with [`execute`](AtParser::execute).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use at_parser_rs::parser::AtParser;
    /// # use at_parser_rs::context::AtContext;
    /// # const SIZE: usize = 64;
    /// # struct MyHandler; impl AtContext<SIZE> for MyHandler {}
    /// let mut parser: AtParser<MyHandler, SIZE> = AtParser::new();
    /// // parser has no commands yet; execute() will return Err(UnknownCommand)
    /// ```
    pub const fn new() -> Self {
        Self { commands: & mut [] }
    }

    /// Register the commands that this parser will dispatch.
    ///
    /// The slice maps each AT command name to a mutable reference to its
    /// handler.  Command names are matched verbatim and case-sensitively
    /// against the prefix of the input string (before any suffix such as
    /// `?`, `=?`, or `=<args>`).
    ///
    /// # Arguments
    ///
    /// * `commands` — mutable slice of `(&'static str, &'static str, &mut T)` triples
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use at_parser_rs::parser::AtParser;
    /// # use at_parser_rs::context::AtContext;
    /// # use at_parser_rs::{AtResult, AtError};
    /// # use osal_rs::utils::Bytes;
    /// # const SIZE: usize = 64;
    /// struct PingModule;
    /// impl AtContext<SIZE> for PingModule {
    ///     fn exec(&mut self) -> AtResult<'_, SIZE> { Ok(Bytes::from_str("PONG")) }
    /// }
    ///
    /// let mut ping = PingModule;
    /// let mut parser: AtParser<PingModule, SIZE> = AtParser::new();
    ///
    /// let commands: &mut [(&str, &str, &mut PingModule)] = &mut [
    ///     ("AT+PING", "+PONG", &mut ping),
    /// ];
    /// parser.set_commands(commands);
    /// ```
    ///
    /// Using trait objects to mix different handler types:
    ///
    /// ```rust,no_run
    /// # use at_parser_rs::parser::AtParser;
    /// # use at_parser_rs::context::AtContext;
    /// # use at_parser_rs::{AtResult, AtError};
    /// # use osal_rs::utils::Bytes;
    /// # const SIZE: usize = 64;
    /// # struct PingModule; impl AtContext<SIZE> for PingModule {}
    /// # struct EchoModule; impl AtContext<SIZE> for EchoModule {}
    /// let mut ping = PingModule;
    /// let mut echo = EchoModule;
    /// let mut parser: AtParser<dyn AtContext<SIZE>, SIZE> = AtParser::new();
    ///
    /// let commands: &mut [(&str, &str, &mut dyn AtContext<SIZE>)] = &mut [
    ///     ("AT+PING", "+PONG", &mut ping),
    ///     ("AT+ECHO", "+ECHO", &mut echo),
    /// ];
    /// parser.set_commands(commands);
    /// ```
    pub fn set_commands(&mut self, commands: &'a mut [(&'static str, &'static str, &'a mut T)]) {
        self.commands = commands;
    }

    /// Parse and execute an AT command string.
    ///
    /// Leading and trailing whitespace is stripped before parsing.
    /// The command name is matched against the registered commands; if found,
    /// the appropriate handler method is called based on the command form
    /// detected from the suffix.
    ///
    /// | Input suffix | Dispatches to |
    /// |---|---|
    /// | *(none)* | [`exec`](crate::context::AtContext::exec) |
    /// | `?` | [`query`](crate::context::AtContext::query) |
    /// | `=?` | [`test`](crate::context::AtContext::test) |
    /// | `=<args>` | [`set`](crate::context::AtContext::set) |
    ///
    /// # Arguments
    ///
    /// * `input` — raw AT command string (e.g. `"AT+CMD?"`, `"AT+CMD=1,2"`)
    ///
    /// # Returns
    ///
    /// * `Ok(Bytes<SIZE>)` — response buffer returned by the matched handler
    /// * `Err(AtError::UnknownCommand)` — no handler found for the command name
    /// * `Err(AtError::NotSupported)` — handler found but the requested form is not implemented
    /// * `Err(AtError::InvalidArgs)` — handler returned an argument error
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use at_parser_rs::parser::AtParser;
    /// # use at_parser_rs::context::AtContext;
    /// # use at_parser_rs::{Args, AtResult, AtError};
    /// # use osal_rs::utils::Bytes;
    /// # const SIZE: usize = 64;
    /// struct EchoModule { enabled: bool }
    /// impl AtContext<SIZE> for EchoModule {
    ///     fn query(&mut self) -> AtResult<'_, SIZE> {
    ///         Ok(Bytes::from_str(if self.enabled { "1" } else { "0" }))
    ///     }
    ///     fn set(&mut self, at_response: &'static str, args: Args) -> AtResult<'_, SIZE> {
    ///         let value = args.get(0).ok_or(AtError::InvalidArgs)?;
    ///         match value.as_ref() {
    ///             "0" => { self.enabled = false; Ok(Bytes::from_str("OK")) }
    ///             "1" => { self.enabled = true;  Ok(Bytes::from_str("OK")) }
    ///             _ => Err(AtError::InvalidArgs),
    ///         }
    ///     }
    /// }
    ///
    /// let mut echo = EchoModule { enabled: false };
    /// let mut parser: AtParser<EchoModule, SIZE> = AtParser::new();
    /// let commands: &mut [(&str, &str, &mut EchoModule)] = &mut [("AT+ECHO", "+ECHO", &mut echo)];
    /// parser.set_commands(commands);
    ///
    /// assert!(parser.execute("AT+ECHO=1").is_ok());   // sets echo on
    /// assert!(parser.execute("AT+ECHO?").is_ok());    // queries state
    /// assert!(parser.execute("AT+UNKNOWN").is_err()); // Err(UnknownCommand)
    /// assert!(parser.execute("AT+ECHO=9").is_err());  // Err(InvalidArgs)
    /// ```
    pub fn execute<'b>(&'b mut self, input: &'b str) -> AtResult<'b, SIZE> {
        let input = input.trim();
        let (name, form) = parse(input).map_err(|e| ("", e))?;

        // Find the command handler
        let (_, at_response, module) = self.commands
            .iter_mut()
            .find(|(n, _, _)| *n == name)
            .ok_or(("", AtError::UnknownCommand))?;

        // Dispatch to the appropriate handler method
        match form {
            AtForm::Exec => module.exec(at_response),
            AtForm::Query => module.query(at_response),
            AtForm::Test => module.test(at_response),
            AtForm::Set(args) => module.set(at_response, args),
        }
    }
}

/// Parse an AT command string into its name and form.
///
/// Examines the suffix of `input` (after trimming whitespace) to determine
/// which AT command form was requested, then returns the bare command name
/// together with the detected [`AtForm`].
///
/// | Suffix | Resulting form |
/// |---|---|
/// | `=?` | [`AtForm::Test`] |
/// | `?` | [`AtForm::Query`] |
/// | `=<args>` | [`AtForm::Set`] with the text after `=` as raw args |
/// | *(none)* | [`AtForm::Exec`] |
///
/// This function never returns an error; every well-formed AT command string
/// maps to exactly one form.
///
/// # Arguments
///
/// * `input` — pre-trimmed AT command string (trimming is reapplied internally)
///
/// # Returns
///
/// `Ok((command_name, form))` where `command_name` is a slice of `input`
/// with the suffix removed.
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