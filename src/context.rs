use crate::{Args, AtError, AtResult};

/// Trait that defines the context for AT command execution.
/// Implementations of this trait handle the actual logic for each AT command form.
pub trait AtContext {

    /// Execute command (AT+CMD)
    /// This is called when a command is invoked without any suffix.
    fn exec(&self) -> AtResult<'static> {
        Err(AtError::NotSupported)
    }

    /// Query command (AT+CMD?)
    /// This is called to retrieve the current value/state of a command.
    fn query(&mut self) -> AtResult<'static> {
        Err(AtError::NotSupported)
    }
    
    /// Test command (AT+CMD=?)
    /// This is called to check if a command is supported or to get valid parameter ranges.
    fn test(&mut self) -> AtResult<'static> {
        Err(AtError::NotSupported)
    }

    /// Set command (AT+CMD=args)
    /// This is called to set parameters for a command.
    fn set(&mut self, _args: Args) -> AtResult<'static> {
        Err(AtError::NotSupported)
    }

}