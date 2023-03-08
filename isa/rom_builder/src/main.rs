// This program generates the ROM file that represents the combinatorial logic
// needed to decode the CPU's instruction set. It iterates over all possible
// instruction values (0x0000 - 0xFFFF) and produces the appropriate control
// lines for each.
//
// Control lines that need to be set:
//
// Jump Condition        - 3b  Lowest bit
// Write Register        - 1b
// Register Write Select - 1b
// Left Select           - 1b
// Right Select          - 1b
// Source Register       - 4b
// Destination Register  - 4b
// Immediate Format      - 2b
// ALU Operation         - 4b
// Write Memory          - 2b
// Read Memory           - 2b
// Writeback Select      - 2b  Highest bit
//
//                Total = 27b
//
//   Byte3     Byte2     Byte1     Byte0
// |-------| |-------| |-------| |-------|
// xxxx x000 0000 0000 0000 0000 0000 0000
//

use std::fs;
use std::io::prelude::*;

mod isa {
    pub struct InstructionField {
        pub width: u16,
        pub shift: u16,
    }

    pub mod opcodes {
        pub const ALU: u16 = 0b0000;
        pub const LUI: u16 = 0b01; // low 2 bits
        pub const LD: u16 = 0b0010;
        pub const AUIPC: u16 = 0b11; // low 2 bits
        pub const ST: u16 = 0b0100;
        pub const LD8S: u16 = 0b0110;
        pub const IMM: u16 = 0b1000;
        pub const LD8: u16 = 0b1010;
        pub const ST8: u16 = 0b1100;
        pub const JMP: u16 = 0b1110;
    }

    pub mod jumps {
        pub const EQ: u16 = 0b000;
        pub const NE: u16 = 0b001;
        pub const LT: u16 = 0b010;
        pub const GE: u16 = 0b011;
        pub const LTS: u16 = 0b100;
        pub const GES: u16 = 0b101;
        pub const JR: u16 = 0b110;
        pub const JRAL: u16 = 0b111;
    }

    pub mod functions {
        pub const ADD: u16 = 0b0000;
        pub const SUB: u16 = 0b0001;
        pub const ADDC: u16 = 0b0010;
        pub const SUBB: u16 = 0b0011;
        pub const AND: u16 = 0b0100;
        pub const OR: u16 = 0b0101;
        pub const XOR: u16 = 0b0110;
        pub const SHL: u16 = 0b0111;
        pub const LSR: u16 = 0b1000;
        pub const ASR: u16 = 0b1001;
        // 0b1010;
        // 0b1011;
        // 0b1100;
        pub const CMP: u16 = 0b1101;
        pub const MOV: u16 = 0b1110;
        pub const SUB_FUNCTION: u16 = 0b1111;
    }

    pub mod sub_functions {
        pub const NOT: u16 = 0b0000;
        pub const NEG: u16 = 0b0001;
        pub const NEGB: u16 = 0b0010;
        // 0b0011 - 0b1110
        pub const CONTROL_FUNCTION: u16 = 0b1111;
    }

    pub mod control_functions {
        pub const NOP: u16 = 0b0000;
        pub const BREAK: u16 = 0b0001;
        pub const HALT: u16 = 0b0010;
        pub const ERR: u16 = 0b0011;
        pub const SYS: u16 = 0b0100;
        pub const SRET: u16 = 0b0101;
    }

    pub mod fields {
        use crate::isa::InstructionField;
        type IF = InstructionField;

        pub const OPCODE: IF = InstructionField { width: 4, shift: 0 };
        pub const SOURCE: IF = InstructionField {
            width: 4,
            shift: 12,
        };
        pub const DESTINATION: IF = InstructionField { width: 4, shift: 8 };
        pub const ALU_FUNCTION: IF = InstructionField { width: 4, shift: 4 };
        pub const IMM_FUNCTION: IF = InstructionField {
            width: 4,
            shift: 12,
        };
        pub const SUB_FUNCTION: IF = InstructionField {
            width: 4,
            shift: 12,
        };
        pub const CTRL_FUNCTION: IF = InstructionField { width: 4, shift: 8 };
        pub const JUMP_TYPE: IF = InstructionField { width: 3, shift: 8 };
    }
}

mod ctrl {
    pub const DEFAULT_REGISTER: u32 = 0b0000;
    pub const LINK_REGISTER: u32 = 0b1111;

    pub struct ControlLines {
        pub condition: u32,
        pub register_write: bool,
        pub register_write_select: RegisterWriteSelect,
        pub left: LeftSelect,
        pub right: RightSelect,
        pub source: u32,
        pub destination: u32,
        pub immediate: u32,
        pub function: u32,
        pub memory_write: bool,
        pub memory_read: u32,
        pub write_back: u32,
    }

    // TODO: How to accomplish this default behavior with the Instruction as a parameter
    // so that the source and destination registers can be pulled out of the instruction
    // automatically since the location of those values never change.
    impl Default for ControlLines {
        fn default() -> Self {
            ControlLines {
                condition: conditions::NEVER,
                register_write: false,
                register_write_select: RegisterWriteSelect::Destination,
                left: LeftSelect::Source,
                right: RightSelect::Destination,
                source: DEFAULT_REGISTER,
                destination: DEFAULT_REGISTER,
                immediate: immediate_format::FOUR,
                function: functions::MOV,
                memory_write: false,
                memory_read: memory_read::WORD,
                write_back: write_back::ALU_RESULT,
            }
        }
    }

    pub struct ControlField {
        pub shift: u32,
    }

    pub mod fields {
        use crate::ctrl::ControlField;
        type CF = ControlField;

        pub const CONDITION: CF = ControlField { shift: 0 };
        pub const REGISTER_WRITE: CF = ControlField { shift: 3 };
        pub const REGISTER_WRITE_SELECT: CF = ControlField { shift: 4 };
        pub const LEFT: CF = ControlField { shift: 5 };
        pub const RIGHT: CF = ControlField { shift: 6 };
        pub const SOURCE: CF = ControlField { shift: 7 };
        pub const DESTINATION: CF = ControlField { shift: 11 };
        pub const IMMEDIATE: CF = ControlField { shift: 15 };
        pub const FUNCTION: CF = ControlField { shift: 17 };
        pub const MEMORY_WRITE: CF = ControlField { shift: 21 };
        pub const MEMORY_READ: CF = ControlField { shift: 23 };
        pub const WRITE_BACK: CF = ControlField { shift: 25 };
    }

    pub mod functions {
        pub const ADD: u32 = 0b0000;
        pub const SUB: u32 = 0b0001;
        pub const ADDC: u32 = 0b0010;
        pub const SUBB: u32 = 0b0011;
        pub const AND: u32 = 0b0100;
        pub const OR: u32 = 0b0101;
        pub const XOR: u32 = 0b0110;
        pub const SHL: u32 = 0b0111;
        pub const LSR: u32 = 0b1000;
        pub const ASR: u32 = 0b1001;
        pub const MOV: u32 = 0b1010;
        pub const RIGHT: u32 = 0b1011;
        pub const NOT: u32 = 0b1100;
        pub const NEG: u32 = 0b1101;
        pub const NEGB: u32 = 0b1110;
    }

    pub mod conditions {
        pub const NEVER: u32 = 0b000;
        pub const EQ: u32 = 0b001;
        pub const NE: u32 = 0b010;
        pub const LT: u32 = 0b011;
        pub const GE: u32 = 0b100;
        pub const LTS: u32 = 0b101;
        pub const GES: u32 = 0b110;
        pub const ALWAYS: u32 = 0b111;
    }

    pub enum LeftSelect {
        Source,
        ProgramCounter,
    }

    pub enum RightSelect {
        Destination,
        Immediate,
    }

    pub enum RegisterWriteSelect {
        Destination,
        Source,
    }

    pub mod immediate_format {
        pub const FOUR: u32 = 0b00;
        pub const SIX: u32 = 0b01;
        pub const NINE: u32 = 0b10;
        pub const TEN: u32 = 0b11;
    }

    pub mod memory_read {
        pub const SIGN_EXTEND: u32 = 0b00;
        pub const WORD: u32 = 0b10;
        pub const ZERO_EXTEND: u32 = 0b11;
    }

    pub mod write_back {
        pub const MEMORY_READ: u32 = 0b00;
        pub const ALU_RESULT: u32 = 0b01;
        pub const PROGRAM_COUNTER: u32 = 0b10;
    }

    impl Into<u32> for LeftSelect {
        fn into(self) -> u32 {
            match self {
                LeftSelect::Source => 0,
                LeftSelect::ProgramCounter => 1,
            }
        }
    }

    impl Into<u32> for RightSelect {
        fn into(self) -> u32 {
            match self {
                RightSelect::Destination => 0,
                RightSelect::Immediate => 1,
            }
        }
    }

    impl Into<u32> for RegisterWriteSelect {
        fn into(self) -> u32 {
            match self {
                RegisterWriteSelect::Destination => 0,
                RegisterWriteSelect::Source => 1,
            }
        }
    }
}

use ctrl::ControlLines;
use ctrl::LeftSelect;
use ctrl::RightSelect;

// Extract bits from an instruction.
fn get_bits(inst: u16, count: u16, shift: u16) -> u16 {
    const BASE: u16 = 2;

    let n = BASE.pow(count.into()) - 1;

    (inst >> shift) & n
}

fn get_field(inst: u16, field: isa::InstructionField) -> u16 {
    get_bits(inst, field.width, field.shift) as u16
}

// Loop over all instruction permutations 0..2^16, decode them into their
// representative control lines, and write them to appropriate decode
// ROM binaries.
fn main() -> Result<(), std::io::Error> {
    let control_words: Vec<u32> = (0..=u16::MAX)
        .into_iter()
        .map(|inst| decode_instruction(inst))
        .collect();

    write_roms(control_words)
}

// Given an Instruction, produce the ControlWord that executes it.
fn decode_instruction(inst: u16) -> u32 {
    let four_bits = get_field(inst, isa::fields::OPCODE);
    let two_bits = get_bits(inst, 2, 0);

    match two_bits {
        isa::opcodes::LUI => decode_lui(inst),
        isa::opcodes::AUIPC => decode_auipc(inst),
        _ => match four_bits {
            isa::opcodes::ALU => decode_alu(inst),
            isa::opcodes::IMM => decode_imm(inst),
            isa::opcodes::LD => decode_ld(inst),
            isa::opcodes::LD8 => decode_ld8(inst),
            isa::opcodes::LD8S => decode_ld8s(inst),
            isa::opcodes::ST => decode_st(inst),
            isa::opcodes::ST8 => decode_st8(inst),
            isa::opcodes::JMP => decode_jmp(inst),
            _ => unreachable!(),
        },
    }
}

fn write_roms(control_words: Vec<u32>) -> std::io::Result<()> {
    let mut bytes = Vec::new();

    for word in control_words {
        for i in 0..4 {
            let byte = ((word & (255 << (i * 8))) >> (i * 8)) as u8;
            bytes.push(byte);
        }
    }

    fs::File::create("../bwb16_control_rom.bin")?.write_all(&bytes)
}

fn encode_control_word(lines: ControlLines) -> u32 {
    let register_write = lines.register_write as u32;
    let memory_write = lines.memory_write as u32;

    let left: u32 = lines.left.into();
    let right: u32 = lines.right.into();
    let register_write_select: u32 = lines.register_write_select.into();

    (lines.condition << ctrl::fields::CONDITION.shift)
        | (register_write << ctrl::fields::REGISTER_WRITE.shift)
        | (register_write_select << ctrl::fields::REGISTER_WRITE_SELECT.shift)
        | (left << ctrl::fields::LEFT.shift)
        | (right << ctrl::fields::RIGHT.shift)
        | (lines.source << ctrl::fields::SOURCE.shift)
        | (lines.destination << ctrl::fields::DESTINATION.shift)
        | (lines.immediate << ctrl::fields::IMMEDIATE.shift)
        | (lines.function << ctrl::fields::FUNCTION.shift)
        | (memory_write << ctrl::fields::MEMORY_WRITE.shift)
        | (lines.memory_read << ctrl::fields::MEMORY_READ.shift)
        | (lines.write_back << ctrl::fields::WRITE_BACK.shift)
}

// Produce the ControlWord for an Instruction with opcode ALU
fn decode_alu(inst: u16) -> u32 {
    let source = get_field(inst, isa::fields::SOURCE) as u32;
    let destination = get_field(inst, isa::fields::DESTINATION) as u32;
    let encoded_function = get_field(inst, isa::fields::ALU_FUNCTION);
    let encoded_sub_function = get_field(inst, isa::fields::SUB_FUNCTION);

    let decoded_function = match encoded_function {
        isa::functions::ADD => ctrl::functions::ADD,
        isa::functions::SUB => ctrl::functions::SUB,
        isa::functions::ADDC => ctrl::functions::ADDC,
        isa::functions::SUBB => ctrl::functions::SUBB,
        isa::functions::AND => ctrl::functions::AND,
        isa::functions::OR => ctrl::functions::OR,
        isa::functions::XOR => ctrl::functions::XOR,
        isa::functions::SHL => ctrl::functions::SHL,
        isa::functions::LSR => ctrl::functions::LSR,
        isa::functions::ASR => ctrl::functions::ASR,
        isa::functions::CMP => ctrl::functions::SUB,
        isa::functions::MOV => ctrl::functions::MOV,
        isa::functions::SUB_FUNCTION => match encoded_sub_function {
            isa::sub_functions::NOT => ctrl::functions::NOT,
            isa::sub_functions::NEG => ctrl::functions::NEG,
            isa::sub_functions::NEGB => ctrl::functions::NEGB,
            isa::sub_functions::CONTROL_FUNCTION => return decode_control(inst),
            _ => ctrl::functions::MOV,
        },
        _ => ctrl::functions::MOV,
    };

    let is_mv_encoded = encoded_function == isa::functions::MOV;
    let is_mv_decoded = decoded_function == ctrl::functions::MOV;

    let register_write = if is_mv_decoded && !is_mv_encoded {
        false
    } else {
        encoded_function != isa::functions::CMP
    };

    encode_control_word(ControlLines {
        register_write,
        source,
        destination,
        function: decoded_function,
        ..Default::default()
    })
}

// Produce the ControlWord for a special purpose Instruction
fn decode_control(inst: u16) -> u32 {
    // Currently treating all as NOP
    // Most only have special functionality in emulation
    // Others are not implemented in hardware yet
    let function = get_field(inst, isa::fields::CTRL_FUNCTION);

    let _dummy = match function {
        isa::control_functions::NOP => (),
        isa::control_functions::BREAK => (),
        isa::control_functions::HALT => (),
        isa::control_functions::ERR => (),
        isa::control_functions::SYS => (),
        isa::control_functions::SRET => (),
        _ => (),
    };

    encode_control_word(ControlLines::default())
}

// Produce the ControlWord for an Instruction with opcode IMM
fn decode_imm(inst: u16) -> u32 {
    let source = get_field(inst, isa::fields::SOURCE) as u32;
    let destination = source;

    let encoded_function = get_field(inst, isa::fields::IMM_FUNCTION);
    let top_two = get_bits(encoded_function, 2, 14);

    let (immediate, operation) = match top_two {
        0b00 => (ctrl::immediate_format::SIX as u32, ctrl::functions::ADD),
        0b01 => (ctrl::immediate_format::SIX as u32, ctrl::functions::SUB),
        0b10 => return decode_jal(inst),
        0b11 => match encoded_function & 0b11 {
            0b00 => (ctrl::immediate_format::FOUR as u32, ctrl::functions::SHL),
            0b01 => (ctrl::immediate_format::FOUR as u32, ctrl::functions::LSR),
            0b10 => (ctrl::immediate_format::FOUR as u32, ctrl::functions::ASR),
            0b11 => (ctrl::immediate_format::FOUR as u32, ctrl::functions::AND),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };

    let register_write = top_two != 0b01;

    encode_control_word(ControlLines {
        register_write,
        right: RightSelect::Immediate,
        source,
        destination,
        immediate,
        function: operation,
        ..Default::default()
    })
}

// Produce the ControlWord for the JAL Instruction
fn decode_jal(_inst: u16) -> u32 {
    encode_control_word(ControlLines {
        register_write: true,
        register_write_select: ctrl::RegisterWriteSelect::Source,
        left: LeftSelect::ProgramCounter,
        right: RightSelect::Immediate,
        source: ctrl::LINK_REGISTER,
        destination: ctrl::LINK_REGISTER,
        immediate: ctrl::immediate_format::SIX,
        function: ctrl::functions::ADD,
        memory_read: ctrl::memory_read::SIGN_EXTEND,
        write_back: ctrl::write_back::PROGRAM_COUNTER,
        ..Default::default()
    })
}

// Produce the ControlWord for an Instruction with opcode LD
fn decode_ld(inst: u16) -> u32 {
    let source = get_field(inst, isa::fields::SOURCE) as u32;
    let destination = get_field(inst, isa::fields::DESTINATION) as u32;

    encode_control_word(ControlLines {
        register_write: true,
        right: RightSelect::Immediate,
        source,
        destination,
        function: ctrl::functions::ADD,
        write_back: ctrl::write_back::MEMORY_READ,
        ..Default::default()
    })
}

// Produce the ControlWord for an Instruction with opcode LD8
fn decode_ld8(inst: u16) -> u32 {
    let source = get_field(inst, isa::fields::SOURCE) as u32;
    let destination = get_field(inst, isa::fields::DESTINATION) as u32;

    encode_control_word(ControlLines {
        register_write: true,
        right: RightSelect::Immediate,
        source,
        destination,
        function: ctrl::functions::ADD,
        memory_read: ctrl::memory_read::ZERO_EXTEND,
        write_back: ctrl::write_back::MEMORY_READ,
        ..Default::default()
    })
}

// Produce the ControlWord for an Instruction with opcode ST
fn decode_st(inst: u16) -> u32 {
    let source = get_field(inst, isa::fields::SOURCE) as u32;
    let destination = get_field(inst, isa::fields::DESTINATION) as u32;

    encode_control_word(ControlLines {
        right: RightSelect::Immediate,
        source,
        destination,
        function: ctrl::functions::ADD,
        memory_write: true,
        memory_read: ctrl::memory_read::ZERO_EXTEND,
        write_back: ctrl::write_back::MEMORY_READ,
        ..Default::default()
    })
}

// Produce the ControlWord for an Instruction with opcode ST8
fn decode_st8(inst: u16) -> u32 {
    let source = get_field(inst, isa::fields::SOURCE) as u32;
    let destination = get_field(inst, isa::fields::DESTINATION) as u32;

    encode_control_word(ControlLines {
        right: RightSelect::Immediate,
        source,
        destination,
        function: ctrl::functions::ADD,
        memory_write: true,
        memory_read: ctrl::memory_read::ZERO_EXTEND,
        write_back: ctrl::write_back::MEMORY_READ,
        ..Default::default()
    })
}

// Produce the ControlWord for an Instruction with opcode LD8S
fn decode_ld8s(inst: u16) -> u32 {
    let source = get_field(inst, isa::fields::SOURCE) as u32;
    let destination = get_field(inst, isa::fields::DESTINATION) as u32;

    encode_control_word(ControlLines {
        condition: ctrl::conditions::NEVER,
        register_write: false,
        register_write_select: ctrl::RegisterWriteSelect::Destination,
        left: LeftSelect::Source,
        right: RightSelect::Immediate,
        source: source,
        destination: destination,
        immediate: ctrl::immediate_format::FOUR,
        function: ctrl::functions::ADD,
        memory_write: true,
        memory_read: ctrl::memory_read::SIGN_EXTEND,
        write_back: ctrl::write_back::MEMORY_READ,
    })
}

// Produce the ControlWord for an Instruction with opcode JMP
fn decode_jmp(inst: u16) -> u32 {
    let jump_kind = get_field(inst, isa::fields::JUMP_TYPE);

    let condition = match jump_kind {
        isa::jumps::EQ => ctrl::conditions::EQ,
        isa::jumps::NE => ctrl::conditions::NE,
        isa::jumps::LT => ctrl::conditions::LT,
        isa::jumps::GE => ctrl::conditions::GE,
        isa::jumps::LTS => ctrl::conditions::LTS,
        isa::jumps::GES => ctrl::conditions::GES,
        isa::jumps::JR => ctrl::conditions::ALWAYS,
        isa::jumps::JRAL => ctrl::conditions::ALWAYS,
        _ => unreachable!(),
    };

    let register_write = jump_kind == isa::jumps::JRAL;

    encode_control_word(ControlLines {
        condition,
        register_write,
        left: LeftSelect::ProgramCounter,
        right: RightSelect::Immediate,
        source: ctrl::LINK_REGISTER,
        destination: ctrl::LINK_REGISTER,
        immediate: ctrl::immediate_format::NINE,
        function: ctrl::functions::ADD,
        write_back: ctrl::write_back::PROGRAM_COUNTER,
        ..Default::default()
    })
}

// Produce the ControlWord for an Instruction with opcode LUI
fn decode_lui(inst: u16) -> u32 {
    let destination = get_field(inst, isa::fields::DESTINATION) as u32;
    let source = destination;

    encode_control_word(ControlLines {
        register_write: true,
        right: RightSelect::Immediate,
        source,
        destination,
        immediate: ctrl::immediate_format::TEN,
        function: ctrl::functions::RIGHT,
        ..Default::default()
    })
}

// Produce the ControlWord for an Instruction with opcode AUIPC
fn decode_auipc(inst: u16) -> u32 {
    let destination = get_field(inst, isa::fields::DESTINATION) as u32;
    let source = destination;

    encode_control_word(ControlLines {
        register_write: true,
        left: LeftSelect::ProgramCounter,
        right: RightSelect::Immediate,
        source,
        destination,
        immediate: ctrl::immediate_format::TEN,
        function: ctrl::functions::ADD,
        ..Default::default()
    })
}
