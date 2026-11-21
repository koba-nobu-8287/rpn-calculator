//! Calculation state, commands, and number formatting for the RPN calculator.

mod arithmetic;
mod bitwise;
mod calculator;
mod command;
mod config;
mod error;
mod flags;
mod format;
mod number;
mod register;
mod stack;
mod state;
mod word;

pub use bitwise::{BitEdit, RotateKind};
pub use calculator::Calculator;
pub use command::{Command, ValueUpdate};
pub use config::{
    CalculatorConfig, NumericMode, OnesZeroMode, Radix, SignedMode, StackLift, WordConfig,
};
pub use error::CalcError;
pub use flags::Flags;
pub use format::format_number;
pub use number::Number;
pub use state::CalculatorSnapshot;
pub use word::{Word, WordSize};
