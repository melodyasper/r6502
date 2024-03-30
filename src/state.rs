use std::sync::{Arc, Mutex};
use tabled::Tabled;
use bitflags::bitflags;


bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Tabled)]
    pub struct SystemFlags: u8 {
        const negative = 0b10000000;
        const overflow = 0b01000000;
        const expansion = 0b00100000;
        const break_command = 0b00010000;
        const decimal = 0b00001000;
        const interrupt_disable = 0b00000100;
        const zero = 0b00000010;
        const carry = 0b00000001;
    }
}

impl std::fmt::Display for SystemFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let value = *self;
        // if (value & Self::negative) == Self::negative {
        //     write!(f, "N | ")?;
        // }
        // if (value & Self::overflow) == Self::overflow {
        //     write!(f, "O | ")?;
        // }
        // if (value & Self::expansion) == Self::expansion {
        //     write!(f, "E | ")?;
        // }
        // if (value & Self::break_command) == Self::break_command {
        //     write!(f, "B | ")?;
        // }
        // if (value & Self::decimal) == Self::decimal {
        //     write!(f, "D | ")?;
        // }
        // if (value & Self::interrupt_disable) == Self::interrupt_disable {
        //     write!(f, "I | ")?;
        // }
        // if (value & Self::zero) == Self::zero {
        //     write!(f, "Z | ")?;
        // }
        // if (value & Self::carry) == Self::carry {
        //     write!(f, "C | ")?;
        // }
        bitflags::parser::to_writer(self, f)?;
        Ok(())
        // write!(f, "]")
    }
    
}

// Impl blocks can be added to flags types
impl SystemFlags {
    pub fn as_u8(&self) -> u8 {
        self.bits()
    }
}
impl From<u8> for SystemFlags {
    fn from(value: u8) -> Self {
        Self::from_bits_retain(value)
    }
}


#[derive(Debug, PartialEq, Eq, Tabled, Clone)]
pub enum SystemAction {
    // You can either read or write a U8 value.
    READ,
    WRITE,
}

impl std::fmt::Display for SystemAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::READ => {
                write!(f, "read")
            },
            Self::WRITE=> {
                write!(f, "write")
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Tabled, Clone)]
pub struct SystemCycle {
    pub address: u16,
    pub value: u8,
    pub action: SystemAction,
}


impl std::fmt::Display for SystemCycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} from {} with value {} ", self.action, self.address, self.value)
    }
}

#[derive(Debug, PartialEq, Eq, Tabled, Clone)]
pub struct SystemState {
    pub running: bool,
    pub pc: u16,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    // Stack Pointer
    // The processor supports a 256 byte stack located between $0100 and $01FF
    pub s: u8,
    pub p: SystemFlags,
    #[tabled(skip)]
    pub cycles: Vec<SystemCycle>,
}

impl Default for SystemState {
    fn default() -> Self {
        Self {
            running: Default::default(),
            pc: Default::default(),
            a: 0,
            x: 0,
            y: 0,
            s: 0,
            p: SystemFlags::default(),
            cycles: Default::default(),
        }
    }
}

pub type SharedSystemState = Arc<Mutex<SystemState>>;

#[derive(Debug)]
pub enum EmulatorError {
    MemoryReadError,
    MemoryWriteError,
    UnimplementedInstruction,
    InvalidInstructionMode,
    ExpectedMemoryPair,
}

impl std::fmt::Display for EmulatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MemoryReadError =>  write!(f, "Memory read error"),
            Self::MemoryWriteError => write!(f, "Memory write error"),
            Self::UnimplementedInstruction => write!(f, "Instruction not implemented"),
            Self::InvalidInstructionMode => write!(f, "Instruction mode is not a valid mode"),
            Self::ExpectedMemoryPair => write!(f, "Memory pair was expected but received None"),
        }
    }
}
