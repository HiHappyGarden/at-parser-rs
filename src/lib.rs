
//! AT Command Parser Library
//!
//! This library provides a flexible parser for AT commands, commonly used in
//! embedded systems and communication devices. It supports no_std environments.

#![cfg_attr(not(feature = "enable_panic"), no_std)]

#[cfg(feature = "enable_panic")]
extern crate alloc;

#[cfg(feature = "osal_rs")]
extern crate osal_rs;

#[cfg(feature = "enable_panic")]
#[global_allocator]
static ALLOC: alloc::alloc::Global = alloc::alloc::Global;

#[cfg(feature = "enable_panic")]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
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