# Parser-RS

A lightweight, `no_std` AT command parser library for embedded Rust applications.

## Overview

Parser-RS provides a flexible framework for implementing AT command interfaces in embedded systems. It supports the standard AT command syntax including execution, query, test, and set operations.

## Features

- `no_std` compatible - suitable for bare-metal and embedded environments
- Zero-allocation parsing using string slices
- Support for all AT command forms:
  - `AT+CMD` - Execute command
  - `AT+CMD?` - Query current value
  - `AT+CMD=?` - Test supported values
  - `AT+CMD=<args>` - Set new value(s)
- Type-safe command registration via traits
- Static command definitions (suitable for embedded/RTOS)

## Command Forms

The parser supports four standard AT command forms:

| Form | Syntax | Purpose | Example |
|------|--------|---------|---------|
| **Execute** | `AT+CMD` | Execute an action | `AT+RST` |
| **Query** | `AT+CMD?` | Get current setting | `AT+ECHO?` |
| **Test** | `AT+CMD=?` | Get supported values | `AT+ECHO=?` |
| **Set** | `AT+CMD=<args>` | Set new value(s) | `AT+ECHO=1` |
## Core Types

### `AtContext` Trait

The main trait for implementing command handlers. Override only the methods your command needs to support:

```rust
pub trait AtContext {
    fn exec(&self) -> AtResult<'static>;
    fn query(&mut self) -> AtResult<'static>;
    fn test(&mut self) -> AtResult<'static>;
    fn set(&mut self, args: Args) -> AtResult<'static>;
}
```

All methods return `NotSupported` by default.

### `AtResult` and `AtError`

```rust
pub type AtResult<'a> = Result<&'a str, AtError>;

pub enum AtError {
    UnknownCommand,   // Command not found
    NotSupported,     // Operation not implemented
    InvalidArgs,      // Invalid argument(s)
}
```

### `Args` Structure

Provides access to comma-separated arguments:

```rust
pub struct Args<'a> {
    pub raw: &'a str,
}

impl<'a> Args<'a> {
    pub fn get(&self, index: usize) -> Option<&'a str>;
}
```

## Usage Examples

### 1. Define Command Modules

Implement the `AtContext` trait for your command handlers:

```rust
use parser_rs::{AtContext, AtResult, AtError, Args};

/// Echo command - returns/sets echo state
pub struct EchoModule {
    pub echo: bool,
}

impl AtContext for EchoModule {
    // Execute: return current echo state
    fn exec(&self) -> AtResult<'static> {
        if self.echo { Ok("1") } else { Ok("0") }
    }

    // Set: enable/disable echo
    fn set(&mut self, args: Args) -> AtResult<'static> {
        let v = args.get(0).ok_or(AtError::InvalidArgs)?;
        self.echo = v == "1";
        Ok("OK")
    }

    // Test: show valid values
    fn test(&mut self) -> AtResult<'static> {
        Ok("0,1")
    }

    // Help: provide command description
    fn help(&self) -> AtResult<'static> {
        Ok("Enable/disable command echo. Usage: AT+ECHO=<0|1>")
    }
}

/// Reset command - executes system reset
pub struct ResetModule;

impl AtContext for ResetModule {
    fn exec(&self) -> AtResult<'static> {
        // Trigger hardware reset
        // reset_system();
        Ok("OK")
    }

    fn help(&self) -> AtResult<'static> {
        Ok("Reset the system. Usage: AT+RST")
    }
}
```

### 2. Define Static Module Instances

For embedded/`no_std` environments, use static mutable variables:

```rust
static mut ECHO: EchoModule = EchoModule { echo: false };
static mut RESET: ResetModule = ResetModule;
```

> **Note**: `static mut` is safe in single-threaded contexts. For RTOS or multi-threaded applications, use `Mutex` or `RefCell` for synchronization.

### 3. Initialize Parser and Register Commands

```rust
use parser_rs::parser::AtParser;

let mut parser = AtParser::new();

unsafe {
    static COMMANDS: &[(&str, &mut dyn AtContext)] = &[
        ("AT+ECHO", &mut ECHO),
        ("AT+RST", &mut RESET),
    ];

    parser.set_commands(COMMANDS);
}
```

### 4. Execute Commands

```rust
// Set echo to enabled
match parser.execute("AT+ECHO=1") {
    Ok(response) => println!("Response: {}", response),  // "OK"
    Err(e) => println!("Error: {:?}", e),
}

// Query current echo state
match parser.execute("AT+ECHO?") {
    Ok(response) => println!("Echo: {}", response),      // "1"
    Err(e) => println!("Error: {:?}", e),
}

// Test supported values
match parser.execute("AT+ECHO=?") {
    Ok(response) => println!("Valid: {}", response),     // "0,1"
    Err(e) => println!("Error: {:?}", e),
}

// Execute reset
match parser.execute("AT+RST") {
    Ok(response) => println!("Response: {}", response),  // "OK"
    Err(e) => println!("Error: {:?}", e),
}

// Get command help
match parser.execute("AT+ECHO=H") {
    Ok(response) => println!("Help: {}", response),      // "Enable/disable command echo..."
    Err(e) => println!("Error: {:?}", e),
}

// Unknown command
match parser.execute("AT+UNKNOWN") {
    Ok(_) => {},
    Err(AtError::UnknownCommand) => println!("Command not found"),
}
```

## Advanced Example: UART Module

```rust
pub struct UartModule {
    pub baudrate: u32,
    pub data_bits: u8,
}

impl AtContext for UartModule {
    // Query: return current configuration
    fn query(&mut self) -> AtResult<'static> {
        // In real code, format to a static buffer
        Ok("115200,8")
    }

    // Set: configure UART
    fn set(&mut self, args: Args) -> AtResult<'static> {
        let baudrate = args.get(0)
            .ok_or(AtError::InvalidArgs)?
            .parse::<u32>()
            .map_err(|_| AtError::InvalidArgs)?;
        
        let data_bits = args.get(1)
            .ok_or(AtError::InvalidArgs)?
            .parse::<u8>()
            .map_err(|_| AtError::InvalidArgs)?;

        if ![7, 8].contains(&data_bits) {
            return Err(AtError::InvalidArgs);
        }

        self.baudrate = baudrate;
        self.data_bits = data_bits;
        
        // Apply configuration to hardware
        // configure_uart(baudrate, data_bits);
        
        Ok("OK")
    }

    // Test: show valid configurations
    fn test(&mut self) -> AtResult<'static> {
        Ok("(9600,19200,38400,57600,115200),(7,8)")
    }

    // Help: provide usage information
    fn help(&self) -> AtResult<'static> {
        Ok("Configure UART parameters. Usage: AT+UART=<baudrate>,<data_bits>")
    }
}
```

Usage:
```rust
parser.execute("AT+UART=?");        // "(9600,19200,38400,57600,115200),(7,8)"
parser.execute("AT+UART=115200,8"); // "OK"
parser.execute("AT+UART?");         // "115200,8"
parser.execute("AT+UART=H");        // "Configure UART parameters..."
```

## Parsing Arguments

The `Args` structure provides a simple interface for accessing comma-separated arguments:

```rust
fn set(&mut self, args: Args) -> AtResult<'static> {
    let arg0 = args.get(0).ok_or(AtError::InvalidArgs)?;
    let arg1 = args.get(1).ok_or(AtError::InvalidArgs)?;
    let arg2 = args.get(2); // Optional argument
    
    // Process arguments...
    Ok("OK")
}
```

For numeric arguments:
```rust
let value = args.get(0)
    .ok_or(AtError::InvalidArgs)?
    .parse::<i32>()
    .map_err(|_| AtError::InvalidArgs)?;
```

## Thread Safety

### Single-threaded (bare-metal)
```rust
static mut MODULE: MyModule = MyModule::new();
// Safe in single-threaded context
```

### Multi-threaded (RTOS)
```rust
use core::cell::RefCell;
use osal_rs::sync::Mutex;

static MODULE: Mutex<RefCell<MyModule>> = Mutex::new(RefCell::new(MyModule::new()));
```

## Best Practices

1. **Keep responses static**: Return `&'static str` when possible to avoid allocations
2. **Validate arguments**: Always check argument count and validity before processing
3. **Handle errors gracefully**: Use appropriate `AtError` variants for different failure modes
4. **Document test responses**: Use `test()` to provide clear usage information
5. **Minimize state**: Keep module state simple and thread-safe

## License

This project is licensed under the same terms as the parent project.
