use crate::{
    CalcError, CalculatorConfig, CalculatorSnapshot, Command, Number, NumericMode, Radix,
    RotateKind, StackLift, ValueUpdate, Word, WordConfig,
    arithmetic::{self, ArithmeticOp, UnaryOp},
    bitwise::{self, BitwiseResult, LogicOp, ShiftOp},
    state::CalculatorState,
};

/// Owns the stack, registers, flags, and numeric configuration.
#[derive(Clone)]
pub struct Calculator {
    state: CalculatorState,
    initial_config: CalculatorConfig,
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}

impl Calculator {
    pub fn new() -> Self {
        Self::with_config(CalculatorConfig::default()).expect("default calculator configuration")
    }

    pub fn with_config(config: CalculatorConfig) -> Result<Self, CalcError> {
        Ok(Self {
            state: CalculatorState::new(&config)?,
            initial_config: config,
        })
    }

    /// Applies one command. A failed command leaves the entire state unchanged.
    pub fn execute(&mut self, command: Command) -> Result<(), CalcError> {
        self.transact(|state| apply_command(state, command))
    }

    /// Stops at the first error, retaining successful preceding commands.
    pub fn execute_all(
        &mut self,
        commands: impl IntoIterator<Item = Command>,
    ) -> Result<(), CalcError> {
        for command in commands {
            self.execute(command)?;
        }
        Ok(())
    }

    /// Updates a value while preserving the current stack-lift setting.
    pub fn update_value(&mut self, update: ValueUpdate) -> Result<(), CalcError> {
        self.transact(|state| match update {
            ValueUpdate::Start(number) => input_value(state, number, true),
            ValueUpdate::ReplaceX(number) => input_value(state, number, false),
        })
    }

    pub fn snapshot(&self) -> CalculatorSnapshot {
        self.state.snapshot()
    }

    pub fn test_bit(&self, index: u8) -> Result<bool, CalcError> {
        self.state.validate(self.initial_config.register_count)?;
        let size = self.state.word_config.size;
        bitwise::validate_index(index, size)?;
        bitwise::test_bit(integer(self.state.stack.x())?, index, size)
    }

    pub fn reset(&mut self) {
        // The register count never changes, so reset can reuse its allocation.
        let config = self.initial_config;
        let state = &mut self.state;
        state.word_config = config.word_config;
        state.radix = config.radix;
        state.numeric_mode = config.numeric_mode;
        state.stack_lift = StackLift::Disabled;
        state.flags = Default::default();
        let zero = state.zero();
        state.stack = crate::stack::RpnStack::new(zero);
        state.registers.clear(zero);
    }

    fn transact(
        &mut self,
        operation: impl FnOnce(&mut CalculatorState) -> Result<(), CalcError>,
    ) -> Result<(), CalcError> {
        let register_count = self.initial_config.register_count;
        self.state.validate(register_count)?;
        let mut candidate = self.state.try_clone()?;
        operation(&mut candidate)?;
        candidate.validate(register_count)?;
        self.state = candidate;
        Ok(())
    }
}

enum LiftEffect {
    Enable,
    Disable,
    Neutral,
}
enum LastXEffect {
    Keep,
    SaveX,
    Clear,
}

struct CommandMetadata {
    lift_effect: LiftEffect,
    last_x_effect: LastXEffect,
}

fn metadata(command: Command, numeric_mode: NumericMode) -> CommandMetadata {
    use Command::*;

    let (lift_effect, last_x_effect) = match command {
        // Dedicated input/stack paths handle these effects.
        Push(_) | Enter | ClearX => (LiftEffect::Neutral, LastXEffect::Keep),
        SwapXY | RollUp | RollDown | Store(_) | Recall(_) | RecallLastX => {
            (LiftEffect::Enable, LastXEffect::Keep)
        }
        ChangeSign if numeric_mode == NumericMode::FloatingPoint => {
            (LiftEffect::Enable, LastXEffect::Keep)
        }
        Add
        | Subtract
        | Multiply
        | Divide
        | Remainder
        | ChangeSign
        | Absolute
        | Sqrt
        | And
        | Or
        | Xor
        | Not
        | ShiftLeft
        | ShiftRight
        | ShiftRightArithmetic
        | RotateLeft
        | RotateRight
        | RotateLeftCarry
        | RotateRightCarry
        | RotateN(_)
        | MaskLeft(_)
        | MaskRight(_)
        | EditBit { .. }
        | CountBits => (LiftEffect::Enable, LastXEffect::SaveX),
        ClearStack => (LiftEffect::Disable, LastXEffect::Clear),
        SetRadix(_)
        | SetWordSize(_)
        | SetSignedMode(_)
        | SetOnesZeroMode(_)
        | SetNumericMode(_)
        | SetFlag { .. }
        | ClearRegisters => (LiftEffect::Neutral, LastXEffect::Keep),
    };
    CommandMetadata {
        lift_effect,
        last_x_effect,
    }
}

fn apply_command(state: &mut CalculatorState, command: Command) -> Result<(), CalcError> {
    use Command::*;

    let x = state.stack.x();
    let y = state.stack.y();
    let config = state.word_config;
    let effects = metadata(command, state.numeric_mode);
    match command {
        Push(number) => {
            input_value(state, number, true)?;
            state.stack_lift = StackLift::Enabled;
        }
        Enter => {
            state.stack.lift();
            state.stack_lift = StackLift::Disabled;
        }
        ClearX => {
            state.stack.set_x(state.zero());
            state.stack_lift = StackLift::Disabled;
        }
        ClearStack => state.stack.clear_values(state.zero()),
        SwapXY => state.stack.swap_xy(),
        RollUp => state.stack.roll_up(),
        RollDown => state.stack.roll_down(),
        Store(index) => state.registers.set(index, x)?,
        Recall(index) => insert_value(state, state.registers.get(index)?, true),
        RecallLastX => insert_value(state, state.stack.last_x(), true),
        ClearRegisters => state.registers.clear(state.zero()),
        SetFlag { index, value } => state.flags.set(index, value)?,
        SetRadix(radix) => {
            if state.numeric_mode == NumericMode::FloatingPoint && radix != Radix::Decimal {
                return Err(CalcError::NumericModeMismatch);
            }
            state.radix = radix;
        }
        SetWordSize(size) => state.reconfigure_word(WordConfig { size, ..config })?,
        SetSignedMode(signed_mode) => state.reconfigure_word(WordConfig {
            signed_mode,
            ..config
        })?,
        SetOnesZeroMode(mode) => state.word_config.ones_zero_mode = mode,
        SetNumericMode(mode) => state.convert_mode(mode)?,
        Add | Subtract | Multiply | Divide | Remainder => {
            let op = match command {
                Add => ArithmeticOp::Add,
                Subtract => ArithmeticOp::Subtract,
                Multiply => ArithmeticOp::Multiply,
                Divide => ArithmeticOp::Divide,
                _ => ArithmeticOp::Remainder,
            };
            let result = arithmetic::eval_binary(op, y, x, &config)?;
            state.stack.drop_stack();
            state.stack.set_x(result.value);
            state.flags.apply(result.flags);
        }
        ChangeSign | Absolute | Sqrt => {
            let op = match command {
                ChangeSign => UnaryOp::ChangeSign,
                Absolute => UnaryOp::Absolute,
                _ => UnaryOp::Sqrt,
            };
            let result = arithmetic::eval_unary(op, x, &config)?;
            state.stack.set_x(result.value);
            state.flags.apply(result.flags);
        }
        And | Or | Xor => {
            let op = match command {
                And => LogicOp::And,
                Or => LogicOp::Or,
                _ => LogicOp::Xor,
            };
            let result = bitwise::logic(op, integer(y)?, Some(integer(x)?), config.size)?;
            state.stack.drop_stack();
            apply_bit_result(state, result);
        }
        Not => {
            let result = bitwise::logic(LogicOp::Not, integer(x)?, None, config.size)?;
            apply_bit_result(state, result);
        }
        ShiftLeft | ShiftRight | ShiftRightArithmetic => {
            let op = match command {
                ShiftLeft => ShiftOp::Left,
                ShiftRight => ShiftOp::RightLogical,
                _ => ShiftOp::RightArithmetic,
            };
            let result = bitwise::shift(op, integer(x)?, &config)?;
            apply_bit_result(state, result);
        }
        RotateLeft | RotateRight | RotateLeftCarry | RotateRightCarry => {
            let kind = match command {
                RotateLeft => RotateKind::Left,
                RotateRight => RotateKind::Right,
                RotateLeftCarry => RotateKind::LeftCarry,
                _ => RotateKind::RightCarry,
            };
            let result = bitwise::rotate(kind, integer(x)?, 1, config.size, state.flags.carry())?;
            apply_bit_result(state, result);
        }
        RotateN(kind) => {
            let count = integer(x)?.to_signed(&config).unsigned_abs();
            if count > u128::from(kind.max_count(config.size)) {
                return Err(CalcError::InvalidCount);
            }
            let result = bitwise::rotate(
                kind,
                integer(y)?,
                count as u8,
                config.size,
                state.flags.carry(),
            )?;
            state.stack.drop_stack();
            apply_bit_result(state, result);
        }
        MaskLeft(count) | MaskRight(count) => {
            bitwise::validate_count(count, config.size)?;
            integer(x)?;
            let result = if matches!(command, MaskLeft(_)) {
                bitwise::mask_left(count, config.size)?
            } else {
                bitwise::mask_right(count, config.size)?
            };
            apply_bit_result(state, result);
        }
        EditBit { index, edit } => {
            bitwise::validate_index(index, config.size)?;
            let result = bitwise::edit_bit(integer(x)?, index, edit, config.size)?;
            apply_bit_result(state, result);
        }
        CountBits => {
            let result = bitwise::count_bits(integer(x)?, config.size);
            apply_bit_result(state, result);
        }
    }
    match effects.last_x_effect {
        LastXEffect::Keep => {}
        LastXEffect::SaveX => state.stack.set_last_x(x),
        LastXEffect::Clear => state.stack.set_last_x(state.zero()),
    }
    match effects.lift_effect {
        LiftEffect::Neutral => {}
        LiftEffect::Enable => state.stack_lift = StackLift::Enabled,
        LiftEffect::Disable => state.stack_lift = StackLift::Disabled,
    }
    Ok(())
}

fn integer(number: Number) -> Result<Word, CalcError> {
    number.as_integer().ok_or(CalcError::NumericModeMismatch)
}

fn input_value(state: &mut CalculatorState, number: Number, lift: bool) -> Result<(), CalcError> {
    number.validate(state.numeric_mode, &state.word_config)?;
    insert_value(state, number, lift);
    Ok(())
}

fn insert_value(state: &mut CalculatorState, number: Number, lift: bool) {
    if lift && state.stack_lift == StackLift::Enabled {
        state.stack.lift();
    }
    state.stack.set_x(number);
}

fn apply_bit_result(state: &mut CalculatorState, result: BitwiseResult) {
    state.stack.set_x(Number::Integer(result.value));
    state.flags.apply(result.flags);
}

#[cfg(test)]
mod tests;
