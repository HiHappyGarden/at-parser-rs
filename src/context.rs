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
 
use crate::{Args, AtError, AtResult};

/// Trait that defines the context for AT command execution.
///
/// Implement this trait for each AT command your device exposes.
/// Each method corresponds to one of the four standard AT command forms.
/// All methods have a default implementation that returns
/// [`AtError::NotSupported`], so you only need to override the forms your
/// command actually needs.
///
/// The const generic `SIZE` defines the capacity (in bytes) of the
/// [`Bytes`](osal_rs::utils::Bytes) response buffer returned by each handler.
/// All handlers registered in the same [`AtParser`](crate::parser::AtParser)
/// must use the same `SIZE`.
///
/// # Example — minimal handler
///
/// ```rust,no_run
/// use at_parser_rs::context::AtContext;
/// use at_parser_rs::{Args, AtResult, AtError};
/// use osal_rs::utils::Bytes;
///
/// const SIZE: usize = 64;
///
/// struct ResetModule;
///
/// impl AtContext<SIZE> for ResetModule {
///     fn exec(&mut self) -> AtResult<'_, SIZE> {
///         // AT+RST performs a system reset
///         Ok(Bytes::from_str("OK"))
///     }
/// }
/// ```
///
/// # Example — full handler with all forms
///
/// ```rust,no_run
/// use at_parser_rs::context::AtContext;
/// use at_parser_rs::{Args, AtResult, AtError};
/// use osal_rs::utils::Bytes;
///
/// const SIZE: usize = 64;
///
/// struct EchoModule { enabled: bool }
///
/// impl AtContext<SIZE> for EchoModule {
///     fn exec(&mut self) -> AtResult<'_, SIZE> {
///         let s = if self.enabled { "ECHO: ON" } else { "ECHO: OFF" };
///         Ok(Bytes::from_str(s))
///     }
///     fn query(&mut self) -> AtResult<'_, SIZE> {
///         Ok(Bytes::from_str(if self.enabled { "1" } else { "0" }))
///     }
///     fn test(&mut self) -> AtResult<'_, SIZE> {
///         Ok(Bytes::from_str("(0,1)"))
///     }
///     fn set(&mut self, args: Args) -> AtResult<'_, SIZE> {
///         match args.get(0) {
///             Some("0") => { self.enabled = false; Ok(Bytes::from_str("OK")) }
///             Some("1") => { self.enabled = true;  Ok(Bytes::from_str("OK")) }
///             _ => Err(AtError::InvalidArgs),
///         }
///     }
/// }
/// ```
pub trait AtContext<const SIZE: usize> {

    /// Execute command (`AT+CMD`)
    ///
    /// Called when the command is invoked without any suffix.
    /// Use this to implement an action that does not require parameters.
    ///
    /// # Returns
    ///
    /// * `Ok(Bytes<SIZE>)` — response to send back to the caller
    /// * `Err(AtError::NotSupported)` — default when not overridden
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use at_parser_rs::context::AtContext;
    /// # use at_parser_rs::{AtResult};
    /// # use osal_rs::utils::Bytes;
    /// # const SIZE: usize = 64;
    /// struct PingModule;
    ///
    /// impl AtContext<SIZE> for PingModule {
    ///     fn exec(&mut self) -> AtResult<'_, SIZE> {
    ///         Ok(Bytes::from_str("PONG"))
    ///     }
    /// }
    /// // AT+PING  →  "PONG"
    /// ```
    fn exec(&mut self) -> AtResult<'_, SIZE> {
        Err(AtError::NotSupported)
    }

    /// Query command (`AT+CMD?`)
    ///
    /// Called to retrieve the current value or state of the command.
    ///
    /// # Returns
    ///
    /// * `Ok(Bytes<SIZE>)` — current value/state
    /// * `Err(AtError::NotSupported)` — default when not overridden
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use at_parser_rs::context::AtContext;
    /// # use at_parser_rs::{AtResult};
    /// # use osal_rs::utils::Bytes;
    /// # const SIZE: usize = 64;
    /// struct VolumeModule { level: u8 }
    ///
    /// impl AtContext<SIZE> for VolumeModule {
    ///     fn query(&mut self) -> AtResult<'_, SIZE> {
    ///         let mut buf = Bytes::new();
    ///         buf.format(core::format_args!("{}", self.level));
    ///         Ok(buf)
    ///     }
    /// }
    /// // AT+VOL?  →  "75"  (if level == 75)
    /// ```
    fn query(&mut self) -> AtResult<'_, SIZE> {
        Err(AtError::NotSupported)
    }
    
    /// Test command (`AT+CMD=?`)
    ///
    /// Called to report whether a command is supported or to return the
    /// valid parameter ranges accepted by [`set`](AtContext::set).
    ///
    /// # Returns
    ///
    /// * `Ok(Bytes<SIZE>)` — human-readable description of valid parameters
    /// * `Err(AtError::NotSupported)` — default when not overridden
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use at_parser_rs::context::AtContext;
    /// # use at_parser_rs::{AtResult};
    /// # use osal_rs::utils::Bytes;
    /// # const SIZE: usize = 64;
    /// struct VolumeModule { level: u8 }
    ///
    /// impl AtContext<SIZE> for VolumeModule {
    ///     fn test(&mut self) -> AtResult<'_, SIZE> {
    ///         Ok(Bytes::from_str("(0-100)"))
    ///     }
    /// }
    /// // AT+VOL=?  →  "(0-100)"
    /// ```
    fn test(&mut self) -> AtResult<'_, SIZE> {
        Err(AtError::NotSupported)
    }

    /// Set command (`AT+CMD=<args>`)
    ///
    /// Called to configure the command with one or more parameters.
    /// Arguments are accessible via [`Args::get`](crate::Args::get) using a
    /// 0-based comma-separated index.
    ///
    /// # Arguments
    ///
    /// * `args` — parsed argument list; use `args.get(n)` to retrieve the
    ///   n-th comma-separated token (0-indexed)
    ///
    /// # Returns
    ///
    /// * `Ok(Bytes<SIZE>)` — confirmation/response
    /// * `Err(AtError::InvalidArgs)` — when arguments are missing or invalid
    /// * `Err(AtError::NotSupported)` — default when not overridden
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use at_parser_rs::context::AtContext;
    /// # use at_parser_rs::{Args, AtResult, AtError};
    /// # use osal_rs::utils::Bytes;
    /// # const SIZE: usize = 64;
    /// struct VolumeModule { level: u8 }
    ///
    /// impl AtContext<SIZE> for VolumeModule {
    ///     fn set(&mut self, args: Args) -> AtResult<'_, SIZE> {
    ///         let val: u8 = args.get(0)
    ///             .ok_or(AtError::InvalidArgs)?
    ///             .parse()
    ///             .map_err(|_| AtError::InvalidArgs)?;
    ///         if val > 100 { return Err(AtError::InvalidArgs); }
    ///         self.level = val;
    ///         Ok(Bytes::from_str("OK"))
    ///     }
    /// }
    /// // AT+VOL=75   →  "OK"      (sets level to 75)
    /// // AT+VOL=200  →  Err(InvalidArgs)
    /// // AT+VOL=     →  Err(InvalidArgs)
    /// ```
    fn set(&mut self, _args: Args) -> AtResult<'_, SIZE> {
        Err(AtError::NotSupported)
    }

}