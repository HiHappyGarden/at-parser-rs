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
//! embedded systems and communication devices. It supports no_std environments.

#![cfg_attr(any(feature = "enable_panic", feature = "osal_rs"), no_std)]

#[cfg(any(feature = "enable_panic", feature = "osal_rs"))]
extern crate alloc;

#[cfg(feature = "osal_rs")]
extern crate osal_rs;

#[cfg(any(feature = "enable_panic", feature = "osal_rs"))]
use core::prelude::rust_2021::*;
use core::iter::Iterator;
use core::option::Option;
use core::result::Result;

#[cfg(feature = "enable_panic")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

pub mod context;
pub mod parser;


/// Error types that can occur during AT command processing
#[derive(Debug)]
pub enum AtError {
    /// The command is not recognized
    UnknownCommand,
    /// The command is recognized but not supported
    NotSupported,
    /// The command arguments are invalid
    InvalidArgs,
}

/// Result type for AT command operations
/// Returns either a static string response or an AtError
pub type AtResult<'a> = Result<&'a str, AtError>;

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
/// Creates a static array of command names and their associated context handlers
#[macro_export]
macro_rules! at_modules {
    (
        $( $name:expr => $module:ident ),* $(,)?
    ) => {
        static COMMANDS: &[(&'static str, &mut dyn AtContext)] = unsafe {
            &[
                $(
                    ($name, &mut $module),
                )*
            ]
        };
    };
}