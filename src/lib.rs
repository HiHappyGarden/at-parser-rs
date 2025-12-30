#![no_std]

extern crate alloc;
extern crate osal_rs;

pub mod context;
pub mod parser;


#[derive(Debug)]
pub enum AtError {
    UnknownCommand,
    NotSupported,
    InvalidArgs,
}

pub type AtResult<'a> = Result<&'a str, AtError>;

pub struct Args<'a> {
    pub raw: &'a str,
}

impl<'a> Args<'a> {
    pub fn get(&self, index: usize) -> Option<&'a str> {
        self.raw.split(',').nth(index)
    }
}


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