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
use parser_rs::context::AtContext;
use parser_rs::{AtResult, AtError, Args};

/// Echo command - returns/sets echo state
pub struct EchoModule {
    pub echo: bool,
}

impl AtContext for EchoModule {
    // Execute: return current echo state
    fn exec(&self) -> AtResult<'static> {
        if self.echo {
            Ok("ECHO: ON")
        } else {
            Ok("ECHO: OFF")
        }
    }

    // Query: return current echo value
    fn query(&mut self) -> AtResult<'static> {
        if self.echo { Ok("1") } else { Ok("0") }
    }

    // Set: enable/disable echo
    fn set(&mut self, args: Args) -> AtResult<'static> {
        let v = args.get(0).ok_or(AtError::InvalidArgs)?;
        match v {
            "0" => {
                self.echo = false;
                Ok("ECHO OFF")
            }
            "1" => {
                self.echo = true;
                Ok("ECHO ON")
            }
            _ => Err(AtError::InvalidArgs),
        }
    }

    // Test: show valid values and usage
    fn test(&mut self) -> AtResult<'static> {
        Ok("Valid values: 0 (OFF), 1 (ON)")
    }
}

/// Reset command - executes system reset
pub struct ResetModule;

impl AtContext for ResetModule {
    fn exec(&self) -> AtResult<'static> {
        // Trigger hardware reset
        // reset_system();
        Ok("OK - System reset")
    }

    fn test(&mut self) -> AtResult<'static> {
        Ok("Reset the system")
    }
}
```

### 2. Create Module Instances

For standard applications, create instances on the stack:

```rust
let mut echo = EchoModule { echo: false };
let mut reset = ResetModule;
```

For embedded/`no_std` environments with `static mut` (single-threaded only):

```rust
static mut ECHO: EchoModule = EchoModule { echo: false };
static mut RESET: ResetModule = ResetModule;
```

> **Note**: `static mut` requires `unsafe` blocks and is only safe in single-threaded contexts. For RTOS or multi-threaded applications, use proper synchronization primitives.

### 3. Initialize Parser and Register Commands

```rust
use parser_rs::parser::AtParser;

let mut parser = AtParser::new();

let commands: &mut [(&str, &mut dyn AtContext)] = &mut [
    ("AT+ECHO", &mut echo),
    ("AT+RST", &mut reset),
];

parser.set_commands(commands);
```

### 4. Execute Commands

```rust
// Execute: show current state
match parser.execute("AT+ECHO") {
    Ok(response) => println!("Response: {}", response),  // "ECHO: OFF"
    Err(e) => println!("Error: {:?}", e),
}

// Test: show valid values
match parser.execute("AT+ECHO=?") {
    Ok(response) => println!("Valid: {}", response),     // "Valid values: 0 (OFF), 1 (ON)"
    Err(e) => println!("Error: {:?}", e),
}

// Set: enable echo
match parser.execute("AT+ECHO=1") {
    Ok(response) => println!("Response: {}", response),  // "ECHO ON"
    Err(e) => println!("Error: {:?}", e),
}

// Query: get current value
match parser.execute("AT+ECHO?") {
    Ok(response) => println!("Echo: {}", response),      // "1"
    Err(e) => println!("Error: {:?}", e),
}

// Execute reset
match parser.execute("AT+RST") {
    Ok(response) => println!("Response: {}", response),  // "OK - System reset"
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

    // Test: show valid configurations and usage
    fn test(&mut self) -> AtResult<'static> {
        Ok("AT+UART=<baudrate>,<data_bits> where baudrate: 9600-115200, data_bits: 7|8")
    }
}
```

Usage:
```rust
parser.execute("AT+UART=?");        // "AT+UART=<baudrate>,<data_bits> where..."
parser.execute("AT+UART=115200,8"); // "OK"
parser.execute("AT+UART?");         // "115200,8"
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

## Examples

The library includes several example files demonstrating different usage patterns:

- **`main_example.rs`** - Complete demonstration with multiple command types (Echo, Reset, Info, LED)
- **`parser_example.rs`** - Shows direct usage of the `AtParser` with comprehensive tests
- **`no_std_basic.rs`** - Basic patterns for no_std/embedded environments
- **`no_std_advanced.rs`** - Advanced patterns with custom error handling
- **`no_std_uart_device.rs`** - UART and device configuration example

Run examples with:
```bash
cargo run --example main_example --no-default-features
cargo run --example parser_example --no-default-features
```

## License

This project is licensed under the same terms as the parent project.
