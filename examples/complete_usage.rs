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
 
//! Complete example demonstrating the AT command parser functionality
//!
//! **Note**: no_std compatible example - designed to compile and run on embedded
//! targets. Demonstrates all AT command forms without std dependency.

#![no_std]
#![no_main]
#![allow(dead_code, unused_variables)]

extern crate at_parser_rs;

use at_parser_rs::context::AtContext;
use at_parser_rs::{Args, AtError, AtResult, Bytes};

const SIZE: usize = 64;

/// Echo command module - manages echo state
pub struct EchoModule {
    pub echo: bool,
}

impl AtContext<SIZE> for EchoModule {
    /// Execute: return current echo state
    fn exec(&self) -> AtResult<'_, SIZE> {
        if self.echo {
            Ok(Bytes::from_str("ECHO: ON"))
        } else {
            Ok(Bytes::from_str("ECHO: OFF"))
        }
    }

    /// Query: return current echo value
    fn query(&mut self) -> AtResult<'_, SIZE> {
        if self.echo {
            Ok(Bytes::from_str("1"))
        } else {
            Ok(Bytes::from_str("0"))
        }
    }

    /// Test: show valid values
    fn test(&mut self) -> AtResult<'_, SIZE> {
        Ok(Bytes::from_str("Valid values: 0 (OFF), 1 (ON)"))
    }

    /// Set: enable/disable echo
    fn set(&mut self, args: Args) -> AtResult<'_, SIZE> {
        let value = args.get(0).ok_or(AtError::InvalidArgs)?;
        match value {
            "0" => {
                self.echo = false;
                Ok(Bytes::from_str("ECHO OFF"))
            }
            "1" => {
                self.echo = true;
                Ok(Bytes::from_str("ECHO ON"))
            }
            _ => Err(AtError::InvalidArgs),
        }
    }
}

/// Reset command module - simulates system reset
pub struct ResetModule;

impl AtContext<SIZE> for ResetModule {
    /// Execute: perform reset
    fn exec(&self) -> AtResult<'_, SIZE> {
        Ok(Bytes::from_str("OK - System reset"))
    }

    /// Test: show command description
    fn test(&mut self) -> AtResult<'_, SIZE> {
        Ok(Bytes::from_str("Reset the system"))
    }
}

/// Info command module - provides system information
pub struct InfoModule {
    pub version: &'static str,
}

impl AtContext<SIZE> for InfoModule {
    /// Execute: return system info
    fn exec(&self) -> AtResult<'_, SIZE> {
        Ok(Bytes::from_str(self.version))
    }

    /// Query: return detailed info
    fn query(&mut self) -> AtResult<'_, SIZE> {
        Ok(Bytes::from_str("AT-Parser-RS v1.0.0 - AT Command Parser Library"))
    }
}

/// LED command module - controls an LED with multiple parameters
pub struct LedModule {
    pub state: bool,
    pub brightness: u8,
}

impl AtContext<SIZE> for LedModule {
    /// Execute: return current LED state
    fn exec(&self) -> AtResult<'_, SIZE> {
        if self.state {
            Ok(Bytes::from_str("LED: ON"))
        } else {
            Ok(Bytes::from_str("LED: OFF"))
        }
    }

    /// Query: return state and brightness
    fn query(&mut self) -> AtResult<'_, SIZE> {
        if self.state {
            Ok(Bytes::from_str("1,100"))
        } else {
            Ok(Bytes::from_str("0,0"))
        }
    }

    /// Test: show usage
    fn test(&mut self) -> AtResult<'_, SIZE> {
        Ok(Bytes::from_str("AT+LED=<state>,<brightness> where state: 0|1, brightness: 0-100"))
    }

    /// Set: change LED state and brightness
    fn set(&mut self, args: Args) -> AtResult<'_, SIZE> {
        let state_str = args.get(0).ok_or(AtError::InvalidArgs)?;
        
        self.state = match state_str {
            "0" => false,
            "1" => true,
            _ => return Err(AtError::InvalidArgs),
        };

        // Optional brightness parameter
        if let Some(brightness_str) = args.get(1) {
            self.brightness = brightness_str
                .parse::<u8>()
                .map_err(|_| AtError::InvalidArgs)?;
            
            if self.brightness > 100 {
                return Err(AtError::InvalidArgs);
            }
        }

        if self.state {
            Ok(Bytes::from_str("LED ON"))
        } else {
            Ok(Bytes::from_str("LED OFF"))
        }
    }
}

/// Helper function to execute a command and ignore the result
fn execute_command(cmd: &str, name: &str, module: &mut dyn AtContext<SIZE>) {
    let result = if let Some(rest) = cmd.strip_prefix(name) {
        if rest.is_empty() {
            module.exec()
        } else if rest == "?" {
            module.query()
        } else if rest == "=?" {
            module.test()
        } else if let Some(args_str) = rest.strip_prefix('=') {
            module.set(Args { raw: args_str })
        } else {
            Err(AtError::InvalidArgs)
        }
    } else {
        Err(AtError::UnknownCommand)
    };
    let _ = result;
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    let mut echo = EchoModule { echo: false };
    let mut reset = ResetModule;
    let mut info = InfoModule { version: "v1.0.0" };
    let mut led = LedModule { state: false, brightness: 0 };

    // INFO
    execute_command("AT+INFO",   "AT+INFO", &mut info);
    execute_command("AT+INFO?",  "AT+INFO", &mut info);

    // ECHO
    execute_command("AT+ECHO",   "AT+ECHO", &mut echo);
    execute_command("AT+ECHO=?", "AT+ECHO", &mut echo);
    execute_command("AT+ECHO=1", "AT+ECHO", &mut echo);
    execute_command("AT+ECHO?",  "AT+ECHO", &mut echo);
    execute_command("AT+ECHO=0", "AT+ECHO", &mut echo);

    // LED
    execute_command("AT+LED=?",   "AT+LED", &mut led);
    execute_command("AT+LED=1",   "AT+LED", &mut led);
    execute_command("AT+LED?",    "AT+LED", &mut led);
    execute_command("AT+LED=1,75","AT+LED", &mut led);
    execute_command("AT+LED=0",   "AT+LED", &mut led);

    // RESET
    execute_command("AT+RST=?", "AT+RST", &mut reset);
    execute_command("AT+RST",   "AT+RST", &mut reset);

    // Error cases
    execute_command("AT+ECHO=2", "AT+ECHO", &mut echo);  // -> Err(InvalidArgs)
    execute_command("AT+INFO=1", "AT+INFO", &mut info);  // -> Err(NotSupported)

    loop {}
}
