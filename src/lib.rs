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
//! use at_parser_rs::{Args, AtResult, AtError, Bytes};
//!
//! const SIZE: usize = 64;
//!
//! // 1. Define a command handler
//! struct EchoModule { echo: bool }
//!
//! impl AtContext<SIZE> for EchoModule {
//!     fn query(&mut self) -> AtResult<SIZE> {
//!         if self.echo { Ok(Bytes::from_str("1")) } else { Ok(Bytes::from_str("0")) }
//!     }
//!     
//!     fn set(&mut self, args: Args) -> AtResult<SIZE> {
//!         match args.get(0) {
//!             Some("0") => { self.echo = false; Ok(Bytes::from_str("OK")) }
//!             Some("1") => { self.echo = true; Ok(Bytes::from_str("OK")) }
//!             _ => Err(AtError::InvalidArgs),
//!         }
//!     }
//! }
//!
//! // 2. Create parser and register commands
//! let mut echo = EchoModule { echo: false };
//! let mut parser: AtParser<EchoModule, SIZE> = AtParser::new();
//!
//! let commands: &mut [(&str, &mut dyn AtContext<SIZE>)] = &mut [
//!     ("AT+ECHO", &mut echo),
//! ];
//! parser.set_commands(commands);
//!
//! // 3. Execute commands
//! parser.execute("AT+ECHO=1");  // Set echo on
//! parser.execute("AT+ECHO?");   // Query current state
//! ```
//!
//! # Features
//!
//! - **`freertos`** (default) - Enable FreeRTOS support via osal-rs
//! - **`posix`** - Enable POSIX support via osal-rs
//! - **`std`** - Enable standard library support via osal-rs
//! - **`disable_panic`** - Pass-through feature to osal-rs for panic handling
//!
//! # Thread Safety
//!
//! The library can be used in single-threaded (bare-metal) or multi-threaded (RTOS)
//! environments. For RTOS, use appropriate synchronization primitives around
//! command handlers (e.g., `Mutex<RefCell<Handler>>`).

#![no_std]

extern crate alloc;
extern crate osal_rs;

use core::iter::Iterator;
use core::option::Option;
use core::result::Result;

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

/// Result type for AT command operations
/// Returns either a `Bytes<SIZE>` response buffer or an `AtError`
pub type AtResult<'a, const SIZE: usize> = Result<Bytes<SIZE>, AtError<'a>>;

/// Structure holding the arguments passed to an AT command
pub struct Args<'a> {
    /// Raw argument string (comma-separated values)
    pub raw: &'a str,
}

impl<'a> Args<'a> {
    /// Get an argument by index (0-based)
    /// Arguments are separated by commas
    pub fn get(&self, index: usize) -> Option<&'a str> {
        self.raw.split(',').nth(index)
    }
}


/// Macro to define AT command modules
///
/// Creates a static array of command names and their associated context handlers.
///
/// # Warning
///
/// This macro uses `unsafe` to create mutable references to static data.
/// It is only suitable for single-threaded embedded contexts.
///
/// # Limitations
///
/// - **Unsafe**: Requires `unsafe` blocks to use
/// - **Single-threaded only**: Not safe for RTOS or multi-threaded environments
/// - **Limited flexibility**: Cannot mix different handler types
///
/// # Example
///
/// ```rust,no_run
/// use at_parser_rs::at_modules;
/// use at_parser_rs::context::AtContext;
///
/// const SIZE: usize = 64;
///
/// static mut ECHO: EchoModule = EchoModule { echo: false };
/// static mut RESET: ResetModule = ResetModule;
///
/// at_modules! {
///     SIZE;
///     "AT+ECHO" => ECHO,
///     "AT+RST" => RESET,
/// }
/// ```
///
/// # Recommended Alternative
///
/// For most use cases, prefer the manual approach:
///
/// ```rust,no_run
/// const SIZE: usize = 64;
/// let commands: &mut [(&str, &mut dyn AtContext<SIZE>)] = &mut [
///     ("AT+ECHO", &mut echo_handler),
///     ("AT+RST", &mut reset_handler),
/// ];
/// parser.set_commands(commands);
/// ```
/// Macro to format an AT response with 1–6 comma-separated parameters.
///
/// # Syntax
///
/// ```rust,ignore
/// at_response!(SIZE, AT_RESP; arg1, arg2, ..., arg6)
/// ```
///
/// - `SIZE` — const usize for the response buffer capacity
/// - `AT_RESP` — the AT response prefix string
/// - `arg1..arg6` — values to append, comma-separated
#[macro_export]
macro_rules! at_response {
    ($size:expr, $at_resp:expr; $a1:expr) => {{
        let mut response = osal_rs::utils::Bytes::<{$size}>::new();
        response.format(format_args!("{}{}", $at_resp, $a1));
        response
    }};
    ($size:expr, $at_resp:expr; $a1:expr, $a2:expr) => {{
        let mut response = osal_rs::utils::Bytes::<{$size}>::new();
        response.format(format_args!("{}{},{}", $at_resp, $a1, $a2));
        response
    }};
    ($size:expr, $at_resp:expr; $a1:expr, $a2:expr, $a3:expr) => {{
        let mut response = osal_rs::utils::Bytes::<{$size}>::new();
        response.format(format_args!("{}{},{},{}", $at_resp, $a1, $a2, $a3));
        response
    }};
    ($size:expr, $at_resp:expr; $a1:expr, $a2:expr, $a3:expr, $a4:expr) => {{
        let mut response = osal_rs::utils::Bytes::<{$size}>::new();
        response.format(format_args!("{}{},{},{},{}", $at_resp, $a1, $a2, $a3, $a4));
        response
    }};
    ($size:expr, $at_resp:expr; $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr) => {{
        let mut response = osal_rs::utils::Bytes::<{$size}>::new();
        response.format(format_args!("{}{},{},{},{},{}", $at_resp, $a1, $a2, $a3, $a4, $a5));
        response
    }};
    ($size:expr, $at_resp:expr; $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr, $a6:expr) => {{
        let mut response = osal_rs::utils::Bytes::<{$size}>::new();
        response.format(format_args!("{}{},{},{},{},{},{}", $at_resp, $a1, $a2, $a3, $a4, $a5, $a6));
        response
    }};
}

#[macro_export]
macro_rules! at_modules {
    (
        $size:expr;
        $( $name:expr => $module:ident ),* $(,)?
    ) => {
        static COMMANDS: &[(&'static str, &mut dyn AtContext<$size>)] = unsafe {
            &[
                $(
                    ($name, &mut $module),
                )*
            ]
        };
    };
}