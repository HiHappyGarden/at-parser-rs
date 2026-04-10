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

//! AT Command Parser Library
//!
//! This library provides a flexible parser for AT commands, commonly used in
//! embedded systems and communication devices. It supports `no_std` environments.
//!
//! # Architecture
//!
//! The library is built around three core components:
//!
//! - **[`AtParser`](parser::AtParser)** - The main parser that processes AT command strings
//! - **[`AtContext`](context::AtContext)** - Trait for implementing command handlers
//! - **[`Args`]** - Structure for accessing command arguments
//!
//! # Command Forms
//!
//! Supports all standard AT command forms:
//! - `AT+CMD` - Execute (action without parameters)
//! - `AT+CMD?` - Query (get current value/state)
//! - `AT+CMD=?` - Test (get supported values/ranges)
//! - `AT+CMD=<args>` - Set (configure with parameters)
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use at_parser_rs::context::AtContext;
//! use at_parser_rs::parser::AtParser;
//! use at_parser_rs::{Args, AtResult, AtError, at_response};
//!
//! const SIZE: usize = 64;
//!
//! // 1. Define a command handler
//! struct EchoModule { echo: bool }
//!
//! impl AtContext<SIZE> for EchoModule {
//!     fn query(&mut self, at_response: &'static str) -> AtResult<SIZE> {
//!         Ok(at_response!(SIZE, at_response; if self.echo { 1u8 } else { 0u8 }))
//!     }
//!
//!     fn set(&mut self, at_response: &'static str, args: Args) -> AtResult<SIZE> {
//!         let value = args.get(0).ok_or((at_response, AtError::InvalidArgs))?;
//!         match value.as_ref() {
//!             "0" => { self.echo = false; Ok(at_response!(SIZE, at_response; "OK")) }
//!             "1" => { self.echo = true;  Ok(at_response!(SIZE, at_response; "OK")) }
//!             _ => Err((at_response, AtError::InvalidArgs)),
//!         }
//!     }
//! }
//!
//! // 2. Create parser and register commands
//! //    Each entry is (at_command, at_response_prefix, handler)
//! let mut echo = EchoModule { echo: false };
//! let mut parser: AtParser<EchoModule, SIZE> = AtParser::new();
//!
//! let commands: &mut [(&str, &str, &mut EchoModule)] = &mut [
//!     ("AT+ECHO", "+ECHO: ", &mut echo),
//! ];
//! parser.set_commands(commands);
//!
//! // 3. Execute commands
//! parser.execute("AT+ECHO=1");  // Set echo on  → Ok(("+ECHO: ", "OK"))
//! parser.execute("AT+ECHO?");   // Query state  → Ok(("+ECHO: ", "1"))
//! ```
//!
//! # Features
//!
//! - **`freertos`** (default) — Enable FreeRTOS support via osal-rs
//! - **`posix`** — Enable POSIX (Linux/macOS) threading support via osal-rs
//! - **`std`** — Enable standard library support via osal-rs
//! - **`disable_panic`** — Pass-through feature to osal-rs; disables the built-in panic handler
//!
//! # Thread Safety
//!
//! The library can be used in single-threaded (bare-metal) or multi-threaded (RTOS)
//! environments. For RTOS, use appropriate synchronization primitives around
//! command handlers (e.g., `Mutex<RefCell<Handler>>`).

#![no_std]

extern crate alloc;
extern crate osal_rs;

use core::option::Option;
use core::result::Result;

use alloc::borrow::Cow;
use alloc::string::String;
use osal_rs::utils::Bytes;

pub mod context;
pub mod parser;


/// Error types that can occur during AT command processing
#[derive(Debug)]
pub enum AtError<'a> {
    /// The command is not recognized
    UnknownCommand,
    /// The command is recognized but not supported
    NotSupported,
    /// The command arguments are invalid
    InvalidArgs,
    /// Unhandled error with description
    Unhandled(&'a str),
    /// Unhandled error with description owned
    UnhandledOwned(String)
}

/// Result type for AT command operations.
///
/// Both the success and the error variant carry the AT response prefix string
/// (`&'static str`) that was registered alongside the command, so callers can always
/// reconstruct the full response line.
///
/// - `Ok((prefix, bytes))` — successful response with the AT prefix and payload
/// - `Err((prefix, error))` — failure with the AT prefix and error kind
pub type AtResult<'a, const SIZE: usize> = Result<(&'static str, Bytes<SIZE>), (&'static str, AtError<'a>)>;

/// Structure holding the arguments passed to an AT command
pub struct Args<'a> {
    /// Raw argument string (comma-separated values)
    pub raw: &'a str,
}

impl<'a> Args<'a> {
    /// Get an argument by index (0-based)
    /// Arguments are separated by commas, except when they are inside
    /// double-quoted strings.
    ///
    /// When an argument is wrapped in double quotes, the outer quotes are
    /// removed from the returned value and escaped quotes (`\"`) are
    /// decoded to `"`.
    pub fn get(&self, index: usize) -> Option<Cow<'a, str>> {
        let (arg, quoted) = self.find(index)?;

        if quoted {
            Some(Self::decode_quoted(arg))
        } else {
            Some(Cow::Borrowed(arg))
        }
    }

    /// Get an argument by index without decoding escape sequences.
    ///
    /// Quoted arguments are still returned without the surrounding quotes.
    pub fn get_raw(&self, index: usize) -> Option<&'a str> {
        self.find(index).map(|(arg, _)| arg)
    }

    /// Backward-compatible alias for [`Args::get`].
    pub fn get_string(&self, index: usize) -> Option<Cow<'a, str>> {
        self.get(index)
    }

    fn find(&self, index: usize) -> Option<(&'a str, bool)> {
        let mut current_index = 0;
        let mut start = 0;
        let mut in_quotes = false;
        let mut escaped = false;

        for (offset, ch) in self.raw.char_indices() {
            if escaped {
                escaped = false;
                continue;
            }

            if in_quotes {
                match ch {
                    '\\' => escaped = true,
                    '"' => in_quotes = false,
                    _ => {}
                }
                continue;
            }

            match ch {
                '"' => in_quotes = true,
                ',' => {
                    if current_index == index {
                        return Some(Self::normalize(&self.raw[start..offset]));
                    }

                    current_index += 1;
                    start = offset + ch.len_utf8();
                }
                _ => {}
            }
        }

        if current_index == index {
            Some(Self::normalize(&self.raw[start..]))
        } else {
            None
        }
    }

    fn normalize(arg: &'a str) -> (&'a str, bool) {
        if let Some(inner) = arg.strip_prefix('"').and_then(|value| value.strip_suffix('"')) {
            (inner, true)
        } else {
            (arg, false)
        }
    }

    fn decode_quoted(arg: &'a str) -> Cow<'a, str> {
        if !arg.contains('\\') {
            return Cow::Borrowed(arg);
        }

        let mut decoded = String::new();
        let mut escaped = false;

        for ch in arg.chars() {
            if escaped {
                match ch {
                    '"' | '\\' => decoded.push(ch),
                    _ => {
                        decoded.push('\\');
                        decoded.push(ch);
                    }
                }
                escaped = false;
                continue;
            }

            if ch == '\\' {
                escaped = true;
                continue;
            }

            decoded.push(ch);
        }

        if escaped {
            decoded.push('\\');
        }

        Cow::Owned(decoded)
    }
}

#[cfg(test)]
mod tests {
    use super::Args;

    #[test]
    fn get_splits_plain_arguments() {
        let args = Args { raw: "foo,bar,baz" };

        assert_eq!(args.get(0).as_deref(), Some("foo"));
        assert_eq!(args.get(1).as_deref(), Some("bar"));
        assert_eq!(args.get(2).as_deref(), Some("baz"));
        assert_eq!(args.get(3), None);
    }

    #[test]
    fn get_keeps_commas_inside_quoted_arguments() {
        let args = Args { raw: "i,\"ciao, sono antonio\",secret" };

        assert_eq!(args.get(0).as_deref(), Some("i"));
        assert_eq!(args.get(1).as_deref(), Some("ciao, sono antonio"));
        assert_eq!(args.get(2).as_deref(), Some("secret"));
    }

    #[test]
    fn get_decodes_escaped_quotes() {
        let args = Args { raw: r#"i,"ciao, sono \"antonio\"",mysecretpassword"# };

        assert_eq!(args.get_raw(1), Some(r#"ciao, sono \"antonio\""#));
        assert_eq!(args.get(1).as_deref(), Some("ciao, sono \"antonio\""));
        assert_eq!(args.get(2).as_deref(), Some("mysecretpassword"));
    }

    #[test]
    fn get_handles_empty_arguments() {
        let args = Args { raw: "first,,\"\",last" };

        assert_eq!(args.get(0).as_deref(), Some("first"));
        assert_eq!(args.get(1).as_deref(), Some(""));
        assert_eq!(args.get(2).as_deref(), Some(""));
        assert_eq!(args.get(3).as_deref(), Some("last"));
    }
}


/// Wraps a value in double-quote characters (`"`).
///
/// Expands to a string literal `"\"<value>\""` suitable for use inside
/// [`at_response!`] arguments when the protocol
/// requires quoted strings.
///
/// # Syntax
///
/// ```rust,ignore
/// at_quoted!(value)
/// ```
///
/// # Examples
///
/// ```rust,no_run
/// use at_parser_rs::at_quoted;
///
/// let q = at_quoted!("hello");   // → `"hello"`
/// let q = at_quoted!(42);        // → `"42"`
/// ```
///
/// Inside an AT response:
///
/// ```rust,no_run
/// use at_parser_rs::{at_response, at_quoted};
///
/// const SIZE: usize = 64;
/// let name = "world";
/// let resp = at_response!(SIZE, "+CMD: "; at_quoted!(name));
/// // resp contains: +CMD: "world"
/// ```
#[macro_export]
macro_rules! at_quoted {
    ($val:expr) => {
        ::core::format_args!("\"{}\"", $val)
    };
}

/// Macro to format an AT response with 1–6 comma-separated parameters.
///
/// Constructs an [`osal_rs::utils::Bytes`] buffer by formatting the given
/// prefix string (`AT_RESP`) followed by the arguments separated by commas.
///
/// # Syntax
///
/// ```rust,ignore
/// at_response!(SIZE, AT_RESP; arg1, arg2, ..., arg6)
/// ```
///
/// - `SIZE` — `const usize` for the response buffer capacity (must match the
///   capacity used by the surrounding [`AtContext`](crate::context::AtContext) impl)
/// - `AT_RESP` — the AT response prefix string literal (e.g. `"+ECHO: "`)
/// - `arg1..arg6` — values to append, comma-separated; any type implementing
///   [`core::fmt::Display`] is accepted, including [`at_quoted!`] expressions
///
/// # Examples
///
/// ```rust,no_run
/// use at_parser_rs::at_response;
///
/// const SIZE: usize = 64;
///
/// // Single boolean argument
/// let resp = at_response!(SIZE, "+ECHO: "; 1u8);
/// // buffer: "+ECHO: 1"
///
/// // Two arguments (state and brightness)
/// let resp = at_response!(SIZE, "+LED: "; 1u8, 75u8);
/// // buffer: "+LED: 1,75"
///
/// // Three arguments
/// let resp = at_response!(SIZE, "+NET: "; "192.168.1.1", 8080u16, 1u8);
/// // buffer: "+NET: 192.168.1.1,8080,1"
/// ```
///
/// Using [`at_quoted!`] inside the response:
///
/// ```rust,no_run
/// use at_parser_rs::{at_response, at_quoted};
///
/// const SIZE: usize = 64;
/// let ssid = "MyNetwork";
/// let resp = at_response!(SIZE, "+WIFI: "; at_quoted!(ssid), -70i8);
/// // buffer: +WIFI: "MyNetwork",-70
/// ```
#[macro_export]
macro_rules! at_response {
    ($size:expr, $at_resp:expr; $a1:expr) => {{
        let mut response = osal_rs::utils::Bytes::<{$size}>::new();
        response.format(format_args!("{}", $a1));
        ($at_resp, response)
    }};
    ($size:expr, $at_resp:expr; $a1:expr, $a2:expr) => {{
        let mut response = osal_rs::utils::Bytes::<{$size}>::new();
        response.format(format_args!("{},{}", $a1, $a2));
        ($at_resp, response)
    }};
    ($size:expr, $at_resp:expr; $a1:expr, $a2:expr, $a3:expr) => {{
        let mut response = osal_rs::utils::Bytes::<{$size}>::new();
        response.format(format_args!("{},{},{}", $a1, $a2, $a3));
        ($at_resp, response)
    }};
    ($size:expr, $at_resp:expr; $a1:expr, $a2:expr, $a3:expr, $a4:expr) => {{
        let mut response = osal_rs::utils::Bytes::<{$size}>::new();
        response.format(format_args!("{},{},{},{}", $a1, $a2, $a3, $a4));
        ($at_resp, response)
    }};
    ($size:expr, $at_resp:expr; $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr) => {{
        let mut response = osal_rs::utils::Bytes::<{$size}>::new();
        response.format(format_args!("{},{},{},{},{}", $a1, $a2, $a3, $a4, $a5));
        ($at_resp, response)
    }};
    ($size:expr, $at_resp:expr; $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr, $a6:expr) => {{
        let mut response = osal_rs::utils::Bytes::<{$size}>::new();
        response.format(format_args!("{},{},{},{},{},{}", $a1, $a2, $a3, $a4, $a5, $a6));
        ($at_resp, response)
    }};
}



/// Declares a static `COMMANDS` table mapping AT command strings to their handlers.
///
/// This macro expands into a `static COMMANDS` binding of type
/// `&[(&'static str, &mut dyn AtContext<SIZE>)]`, which can then be passed to
/// [`AtParser::set_commands`](crate::parser::AtParser::set_commands).
///
/// # Syntax
///
/// ```rust,ignore
/// at_modules! {
///     SIZE;
///     ("AT+CMD1", "+CMD1: ") => HANDLER1,
///     ("AT+CMD2", "+CMD2: ") => HANDLER2,
/// }
/// ```
///
/// - `SIZE` — `const usize` that defines the response buffer capacity (must match the
///   capacity used by [`AtParser`](crate::parser::AtParser) and every [`AtContext`](crate::context::AtContext) impl).
/// - `"AT+CMD"` — the AT command string the parser will match against the input.
/// - `"+CMD: "` — the AT response prefix forwarded to every handler method.
/// - `HANDLER` — a `static mut` variable that implements [`AtContext<SIZE>`](crate::context::AtContext).
///
/// # Safety
///
/// The macro uses an `unsafe` block internally to obtain `&mut` references to
/// `static mut` items.  It is the caller's responsibility to ensure:
///
/// - **Single-threaded access only** — do not call this in a multi-threaded or
///   RTOS context without external synchronisation.
/// - **One call site** — the generated `COMMANDS` symbol is `static`; defining it
///   more than once in the same scope will cause a compile error.
///
/// # Limitations
///
/// - All handlers must implement `AtContext` with the **same** `SIZE` constant.
/// - The generated symbol is always named `COMMANDS`; rename it after expansion
///   if you need multiple tables.
///
/// # Example — basic usage
///
/// ```rust,no_run
/// use at_parser_rs::at_modules;
/// use at_parser_rs::context::AtContext;
/// use at_parser_rs::{Args, AtResult, AtError, at_response};
///
/// const SIZE: usize = 64;
///
/// struct EchoModule { echo: bool }
/// impl AtContext<SIZE> for EchoModule {
///     fn query(&mut self, at_response: &'static str) -> AtResult<SIZE> {
///         Ok(at_response!(SIZE, at_response; if self.echo { 1u8 } else { 0u8 }))
///     }
///     fn set(&mut self, at_response: &'static str, args: Args) -> AtResult<SIZE> {
///         let value = args.get(0).ok_or((at_response, AtError::InvalidArgs))?;
///         match value.as_ref() {
///             "0" => { self.echo = false; Ok(at_response!(SIZE, at_response; "OK")) }
///             "1" => { self.echo = true;  Ok(at_response!(SIZE, at_response; "OK")) }
///             _ => Err((at_response, AtError::InvalidArgs)),
///         }
///     }
/// }
///
/// struct ResetModule;
/// impl AtContext<SIZE> for ResetModule {
///     fn exec(&mut self, at_response: &'static str) -> AtResult<SIZE> {
///         Ok(at_response!(SIZE, at_response; "OK"))
///     }
/// }
///
/// static mut ECHO:  EchoModule  = EchoModule { echo: false };
/// static mut RESET: ResetModule = ResetModule;
///
/// at_modules! {
///     SIZE;
///     ("AT+ECHO", "+ECHO: ") => ECHO,
///     ("AT+RST",  "+RST: ")  => RESET,
/// }
///
/// // COMMANDS is now available in scope:
/// // parser.set_commands(COMMANDS);
/// ```
///
/// # Example — single handler
///
/// ```rust,no_run
/// use at_parser_rs::at_modules;
/// use at_parser_rs::context::AtContext;
/// use at_parser_rs::{AtResult, at_response};
///
/// const SIZE: usize = 32;
///
/// struct PingModule;
/// impl AtContext<SIZE> for PingModule {
///     fn exec(&mut self, at_response: &'static str) -> AtResult<SIZE> {
///         Ok(at_response!(SIZE, at_response; "PONG"))
///     }
/// }
///
/// static mut PING: PingModule = PingModule;
///
/// at_modules! {
///     SIZE;
///     ("AT+PING", "+PING: ") => PING,
/// }
/// ```
///
/// # Recommended alternative
///
/// For multi-type handler tables or when `static mut` is undesirable, prefer the
/// explicit slice approach — it requires no `unsafe` at the call site and allows
/// mixing handler types via trait objects:
///
/// ```rust,no_run
/// use at_parser_rs::context::AtContext;
///
/// const SIZE: usize = 64;
///
/// let mut echo  = EchoModule { echo: false };
/// let mut reset = ResetModule;
///
/// let commands: &mut [(&str, &str, &mut dyn AtContext<SIZE>)] = &mut [
///     ("AT+ECHO", "+ECHO: ", &mut echo),
///     ("AT+RST",  "+RST: ",  &mut reset),
/// ];
/// parser.set_commands(commands);
/// ```
#[macro_export]
macro_rules! at_modules {
    (
        $size:expr;
        $( ($name:expr, $at_resp:expr) => $module:ident ),* $(,)?
    ) => {
        static COMMANDS: &mut [(&'static str, &'static str, &mut dyn AtContext<$size>)] = unsafe {
            &mut [
                $(
                    ($name, $at_resp, &mut $module),
                )*
            ]
        };
    };
}