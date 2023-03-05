// This program generates the ROM file that represents the combinatorial logic
// needed to decode the CPU's instruction set. It iterates over all possible
// instruction values (0x0000 - 0xFFFF) and produces the appropriate control
// lines for each.
//
// Control lines that need to be set:
//
// Jump Condition       - 3b  Lowest bit
// Write Register       - 1b
// Left Select          - 1b
// Right Select         - 1b
// Source Register      - 4b
// Destination Register - 4b
// Immediate Format     - 2b
// ALU Operation        - 4b
// Write Memory         - 2b
// Read Memory          - 2b
// Writeback Select     - 2b  Highest bit
//
//                Total = 26b
//
//   Byte3     Byte2     Byte1     Byte0
// |-------| |-------| |-------| |-------|
// xxxx xxx0 0000 0000 0000 0000 0000 0000
//

use std::fs::File;
use std::io::prelude::*;

type Instruction = u16;
type ControlWord = u32;

// ------------------------------------

struct InstructionField {
    width: Instruction,
    shift: Instruction,
}

const INSTRUCTION_OPCODE: InstructionField = InstructionField {width: 4, shift: 0};
const INSTRUCTION_SOURCE: InstructionField = InstructionField {width: 4, shift: 12};
const INSTRUCTION_DESTINATION: InstructionField = InstructionField {width: 4, shift: 8};
const INSTRUCTION_FUNCTION: InstructionField = InstructionField {width: 4, shift: 4};
const INSTRUCTION_IMMEDIATE_FUNCTION: InstructionField = InstructionField {width: 4, shift: 12};
const INSTRUCTION_JUMP_TYPE: InstructionField = InstructionField {width: 3, shift: 8};
const INSTRUCTION_SUB_FUNCTION: InstructionField = InstructionField {width: 4, shift: 12};
const INSTRUCTION_SPECIAL_FUNCTION: InstructionField = InstructionField {width: 4, shift: 8};

const OPCODE_LUI: Instruction = 0b01;  // Low two
const OPCODE_AUIPC: Instruction = 0b11;
const OPCODE_ALU: Instruction = 0b0000;  // Low four
const OPCODE_IMM: Instruction = 0b1000;
const OPCODE_LD: Instruction = 0b0010;
const OPCODE_LD8: Instruction = 0b1010;
const OPCODE_ST: Instruction = 0b0100;
const OPCODE_ST8: Instruction = 0b1100;
const OPCODE_LD8S: Instruction = 0b0110;
const OPCODE_JMP: Instruction = 0b1110;

const JUMP_TYPE_EQ: u16 = 0b000;
const JUMP_TYPE_NE: u16 = 0b001;
const JUMP_TYPE_LT: u16 = 0b010;
const JUMP_TYPE_GE: u16 = 0b011;
const JUMP_TYPE_LTS: u16 = 0b100;
const JUMP_TYPE_GES: u16 = 0b101;
const JUMP_TYPE_JR: u16 = 0b110;
const JUMP_TYPE_JRAL: u16 = 0b111;

const FUNCTION_ADD: Instruction = 0b0000;
const FUNCTION_SUB: Instruction = 0b0001;
const FUNCTION_ADDC: Instruction = 0b0010;
const FUNCTION_SUBB: Instruction = 0b0011;
const FUNCTION_AND: Instruction = 0b0100;
const FUNCTION_OR: Instruction = 0b0101;
const FUNCTION_XOR: Instruction = 0b0110;
const FUNCTION_SHL: Instruction = 0b0111;
const FUNCTION_LSR: Instruction = 0b1000;
const FUNCTION_ASR: Instruction = 0b1001;
// 0b1010;
// 0b1011;
// 0b1100;
const FUNCTION_CMP: Instruction = 0b1101;
const FUNCTION_MOV: Instruction = 0b1110;
const FUNCTION_SINGLE_OPERAND: Instruction = 0b1111;

const SUB_FUNCTION_NOT: Instruction = 0b0000;
const SUB_FUNCTION_NEG: Instruction = 0b0001;
const SUB_FUNCTION_NEGB: Instruction = 0b0010;
// 0b0011 - 0b1110
const SUB_FUNCTION_SPECIAL: Instruction = 0b1111;

const SPECIAL_FUNCTION_NOP: Instruction = 0b0000;
const SPECIAL_FUNCTION_BREAK: Instruction = 0b0001;
const SPECIAL_FUNCTION_HALT: Instruction = 0b0010;
const SPECIAL_FUNCTION_ERR: Instruction = 0b0011;
const SPECIAL_FUNCTION_SYS: Instruction = 0b0100;
const SPECIAL_FUNCTION_SRET: Instruction = 0b0101;

// ------------------------------------

const CONTROL_LINK_REGISTER: u32 = 0b1111;

struct ControlField {
    _width: ControlWord,
    shift: ControlWord,
}

const CONTROL_CONDITION: ControlField = ControlField {_width: 3, shift: 0};
const CONTROL_REGISTER_WRITE: ControlField = ControlField {_width: 1, shift: 3};
const CONTROL_LEFT: ControlField = ControlField {_width: 1, shift: 4};
const CONTROL_RIGHT: ControlField = ControlField {_width: 1, shift: 5};
const CONTROL_SOURCE: ControlField = ControlField {_width: 4, shift: 6};
const CONTROL_DESTINATION: ControlField = ControlField {_width: 4, shift: 10};
const CONTROL_IMMEDIATE: ControlField = ControlField {_width: 2, shift: 14};
const CONTROL_FUNCTION: ControlField = ControlField {_width: 4, shift: 16};
const CONTROL_MEMORY_WRITE: ControlField = ControlField {_width: 2, shift: 20};
const CONTROL_MEMORY_READ: ControlField = ControlField {_width: 2, shift: 22};
const CONTROL_WRITE_BACK: ControlField = ControlField {_width: 2, shift: 24};

struct FunctionDecoding {
    add: ControlWord,
    sub: ControlWord,
    addc: ControlWord,
    subb: ControlWord,
    and: ControlWord,
    or: ControlWord,
    xor: ControlWord,
    shl: ControlWord,
    lsr: ControlWord,
    asr: ControlWord,
    mv: ControlWord,
    right: ControlWord,
    not: ControlWord,
    neg: ControlWord,
    negb: ControlWord,
}

const FUNCTION_DECODING: FunctionDecoding = FunctionDecoding {
    add: 0b0000,
    sub: 0b0001,
    addc: 0b0010,
    subb: 0b0011,
    and: 0b0100,
    or: 0b0101,
    xor: 0b0110,
    shl: 0b0111,
    lsr: 0b1000,
    asr: 0b1001,
    mv: 0b1010,
    right: 0b1011,
    not: 0b1100,
    neg: 0b1101,
    negb: 0b1110,
};

struct JumpCondition {
    never: ControlWord,
    eq: ControlWord,
    ne: ControlWord,
    lt: ControlWord,
    ge: ControlWord,
    lts: ControlWord,
    ges: ControlWord,
    always: ControlWord,
}

const CONDITION: JumpCondition = JumpCondition {
    never: 0b000,
    eq: 0b001,
    ne: 0b010,
    lt: 0b011,
    ge: 0b100,
    lts: 0b101,
    ges: 0b110,
    always: 0b111,
};

struct MemoryRead {
    sign_extend: ControlWord,
    word: ControlWord,
    zero_extend: ControlWord,
}

const MEMORY_READ: MemoryRead = MemoryRead {
    sign_extend: 0b00,
    zero_extend: 0b10,
    word: 0b11,
};

struct WriteBack {
    memory_read: ControlWord,
    alu_result: ControlWord,
    program_counter: ControlWord,
}

const WRITE_BACK: WriteBack = WriteBack {
    memory_read: 0b00,
    alu_result: 0b01,
    program_counter: 0b10,
};

struct Immediate {
    four: ControlWord,
    six: ControlWord,
    nine: ControlWord,
    ten: ControlWord,
}

const IMMEDIATE: Immediate = Immediate {
    four: 0b00,
    six: 0b01,
    nine: 0b10,
    ten: 0b11,
};

struct LeftSelect {
    source: bool,
    program_counter: bool,
}

const LEFT_SELECT: LeftSelect = LeftSelect {
    source: false,
    program_counter: true,
};

struct RightSelect {
    destination: bool,
    immediate: bool,
}

const RIGHT_SELECT: RightSelect = RightSelect {
    destination: false,
    immediate: true,
};

struct ControlLines {
    condition: ControlWord,
    register_write: bool,
    left: bool,
    right: bool,
    source: ControlWord,
    destination: ControlWord,
    immediate: ControlWord,
    function: ControlWord,
    memory_write: bool,
    memory_read: ControlWord,
    write_back: ControlWord,
}

// ------------------------------------

// Extract bits from an instruction.
fn get_bits(inst: Instruction, count: u16, shift: u16) -> u16 {
    const BASE: u16 = 2;

    let n = BASE.pow(count.into()) - 1;

    (inst >> shift) & n
}

fn get_field(inst: Instruction, field: InstructionField) -> Instruction {
    get_bits(inst, field.width, field.shift) as Instruction
}

// Loop over all instruction permutations 0..2^16, decode them into their
// representative control lines, and write them to appropriate decode
// ROM binaries.
fn main() -> Result<(), std::io::Error>{
    let control_words: Vec<u32> = (0..=u16::MAX)
        .into_iter()
        .map(|inst| decode_instruction(inst))
        .collect();

    write_roms(control_words)
}

// Given an Instruction, produce the ControlWord that executes it.
fn decode_instruction(inst: Instruction) -> ControlWord {
    let four_bits = get_field(inst, INSTRUCTION_OPCODE);
    let two_bits = get_bits(inst, 2, 0);

    match two_bits {
        OPCODE_LUI => decode_lui(inst),
        OPCODE_AUIPC => decode_auipc(inst),
        _ => match four_bits {
            OPCODE_ALU => decode_alu(inst),
            OPCODE_IMM => decode_imm(inst),
            OPCODE_LD => decode_ld(inst),
            OPCODE_LD8 => decode_ld8(inst),
            OPCODE_LD8S => decode_ld8s(inst),
            OPCODE_ST => decode_st(inst),
            OPCODE_ST8 => decode_st8(inst),
            OPCODE_JMP => decode_jmp(inst),
            _ => unreachable!(),
        }
    }
}

fn write_roms(control_words: Vec<ControlWord>) -> std::io::Result<()> {
    for i in 0..4 {
        let file_name = format!("../cpu16_control_rom_{}.bin", i);

        let bytes = &control_words
            .clone()
            .into_iter()
            .map(
                |word| ((word & (255 << (i * 8))) >> (i * 8)) as u8
            )
            .collect::<Vec<u8>>();

        File::create(file_name)?.write_all(bytes)?;
    }

    Ok(())
}

fn encode_control_word(lines: ControlLines) -> ControlWord {
    let register_write = if lines.register_write { 1 } else { 0 };
    let left = if lines.left { 1 } else { 0 };
    let right = if lines.right { 1 } else { 0 };
    let memory_write = if lines.memory_write { 1 } else { 0 };

    (lines.condition << CONTROL_CONDITION.shift) |
    (register_write << CONTROL_REGISTER_WRITE.shift) |
    (left << CONTROL_LEFT.shift) |
    (right << CONTROL_RIGHT.shift) |
    (lines.source << CONTROL_SOURCE.shift) |
    (lines.destination << CONTROL_DESTINATION.shift) |
    (lines.immediate << CONTROL_IMMEDIATE.shift) |
    (lines.function << CONTROL_FUNCTION.shift) |
    (memory_write << CONTROL_MEMORY_WRITE.shift) |
    (lines.memory_read << CONTROL_MEMORY_READ.shift) |
    (lines.write_back << CONTROL_WRITE_BACK.shift)
}

// Produce the ControlWord for an Instruction with opcode ALU
fn decode_alu(inst: Instruction) -> ControlWord {
    let source = get_field(inst, INSTRUCTION_SOURCE) as u32;
    let destination = get_field(inst, INSTRUCTION_DESTINATION) as u32;
    let encoded_function = get_field(inst, INSTRUCTION_FUNCTION);
    let encoded_sub_function = get_field(inst, INSTRUCTION_SUB_FUNCTION);

    let decoded_function = match encoded_function {
        FUNCTION_ADD => FUNCTION_DECODING.add,
        FUNCTION_SUB => FUNCTION_DECODING.sub,
        FUNCTION_ADDC => FUNCTION_DECODING.addc,
        FUNCTION_SUBB => FUNCTION_DECODING.subb,
        FUNCTION_AND => FUNCTION_DECODING.and,
        FUNCTION_OR => FUNCTION_DECODING.or,
        FUNCTION_XOR => FUNCTION_DECODING.xor,
        FUNCTION_SHL => FUNCTION_DECODING.shl,
        FUNCTION_LSR => FUNCTION_DECODING.lsr,
        FUNCTION_ASR => FUNCTION_DECODING.asr,
        FUNCTION_CMP => FUNCTION_DECODING.sub,
        FUNCTION_MOV => FUNCTION_DECODING.mv,
        FUNCTION_SINGLE_OPERAND => match encoded_sub_function {
            SUB_FUNCTION_NOT => FUNCTION_DECODING.not,
            SUB_FUNCTION_NEG => FUNCTION_DECODING.neg,
            SUB_FUNCTION_NEGB => FUNCTION_DECODING.negb,
            SUB_FUNCTION_SPECIAL => return decode_special(inst),
            _ => FUNCTION_DECODING.mv,
        },
        _ => FUNCTION_DECODING.mv,
    };

    let is_mv_encoded = encoded_function == FUNCTION_MOV;
    let is_mv_decoded = decoded_function == FUNCTION_DECODING.mv;

    let register_write = if is_mv_decoded && !is_mv_encoded {
        false
    } else {
        encoded_function != FUNCTION_CMP
    };

    encode_control_word(ControlLines {
        condition: CONDITION.never,
        register_write: register_write,
        left: LEFT_SELECT.source,
        right: RIGHT_SELECT.destination,
        source: source,
        destination: destination,
        immediate: IMMEDIATE.four,
        function: decoded_function,
        memory_write: false,
        memory_read: MEMORY_READ.word,
        write_back: WRITE_BACK.alu_result,
    })
}

// Produce the ControlWord for a special purpose Instruction
fn decode_special(inst: Instruction) -> ControlWord {
    // Currently treating all as NOP
    // Most only have special functionality in emulation
    // Others are not implemented in hardware yet
    let function = get_field(inst, INSTRUCTION_SPECIAL_FUNCTION);

    let _dummy = match function {
        SPECIAL_FUNCTION_NOP => (),
        SPECIAL_FUNCTION_BREAK => (),
        SPECIAL_FUNCTION_HALT => (),
        SPECIAL_FUNCTION_ERR => (),
        SPECIAL_FUNCTION_SYS => (),
        SPECIAL_FUNCTION_SRET => (),
        _ => (),
    };

    encode_control_word(ControlLines {
        condition: CONDITION.never,
        register_write: false,
        left: LEFT_SELECT.source,
        right: RIGHT_SELECT.destination,
        source: 0b0000,
        destination: 0b0000,
        immediate: IMMEDIATE.four,
        function: FUNCTION_DECODING.mv,
        memory_write: false,
        memory_read: MEMORY_READ.word,
        write_back: WRITE_BACK.alu_result,
    })
}

// Produce the ControlWord for an Instruction with opcode IMM
fn decode_imm(inst: Instruction) -> ControlWord {
    let source = get_field(inst, INSTRUCTION_SOURCE) as u32;
    let destination = source;

    let encoded_function = get_field(inst, INSTRUCTION_IMMEDIATE_FUNCTION);
    let top_two = get_bits(encoded_function as u16, 2, 14);

    let (immediate, operation) = match top_two {
        0b00 => (IMMEDIATE.six as u32, FUNCTION_DECODING.add),
        0b01 => (IMMEDIATE.six as u32, FUNCTION_DECODING.sub),
        0b10 => return decode_jal(inst),
        0b11 => match encoded_function & 0b11 {
            0b00 => (IMMEDIATE.four as u32, FUNCTION_DECODING.shl),
            0b01 => (IMMEDIATE.four as u32, FUNCTION_DECODING.lsr),
            0b10 => (IMMEDIATE.four as u32, FUNCTION_DECODING.asr),
            0b11 => (IMMEDIATE.four as u32, FUNCTION_DECODING.and),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };

    let register_write = if top_two == 0b01 { false } else { true };

    encode_control_word(ControlLines {
        condition: CONDITION.never,
        register_write: register_write,
        left: LEFT_SELECT.source,
        right: RIGHT_SELECT.immediate,
        source: source,
        destination: destination,
        immediate: immediate,
        function: operation,
        memory_write: false,
        memory_read: MEMORY_READ.word,
        write_back: WRITE_BACK.alu_result,
    })
}

// Produce the ControlWord for the JAL Instruction
fn decode_jal(_inst: Instruction) -> ControlWord {
    encode_control_word(ControlLines {
        condition: CONDITION.always,
        register_write: true,
        left: LEFT_SELECT.program_counter,
        right: RIGHT_SELECT.immediate,
        source: CONTROL_LINK_REGISTER,
        destination: CONTROL_LINK_REGISTER,
        immediate: IMMEDIATE.six as u32,
        function: FUNCTION_DECODING.add,
        memory_write: false,
        memory_read: MEMORY_READ.sign_extend,
        write_back: WRITE_BACK.program_counter,
    })
}

// Produce the ControlWord for an Instruction with opcode LD
fn decode_ld(inst: Instruction) -> ControlWord {
    let source = get_field(inst, INSTRUCTION_SOURCE) as u32;
    let destination = get_field(inst, INSTRUCTION_DESTINATION) as u32;

    encode_control_word(ControlLines {
        condition: CONDITION.never,
        register_write: true,
        left: LEFT_SELECT.source,
        right: RIGHT_SELECT.immediate,
        source: source,
        destination: destination,
        immediate: IMMEDIATE.four,
        function: FUNCTION_DECODING.add,
        memory_write: false,
        memory_read: MEMORY_READ.word,
        write_back: WRITE_BACK.memory_read,
    })
}

// Produce the ControlWord for an Instruction with opcode LD8
fn decode_ld8(inst: Instruction) -> ControlWord {
    let source = get_field(inst, INSTRUCTION_SOURCE) as u32;
    let destination = get_field(inst, INSTRUCTION_DESTINATION) as u32;

    encode_control_word(ControlLines {
        condition: CONDITION.never,
        register_write: true,
        left: LEFT_SELECT.source,
        right: RIGHT_SELECT.immediate,
        source: source,
        destination: destination,
        immediate: IMMEDIATE.four,
        function: FUNCTION_DECODING.add,
        memory_write: false,
        memory_read: MEMORY_READ.zero_extend,
        write_back: WRITE_BACK.memory_read,
    })
}

// Produce the ControlWord for an Instruction with opcode ST
fn decode_st(inst: Instruction) -> ControlWord {
    let source = get_field(inst, INSTRUCTION_SOURCE) as u32;
    let destination = get_field(inst, INSTRUCTION_DESTINATION) as u32;

    encode_control_word(ControlLines {
        condition: CONDITION.never,
        register_write: false,
        left: LEFT_SELECT.source,
        right: RIGHT_SELECT.immediate,
        source: source,
        destination: destination,
        immediate: IMMEDIATE.four,
        function: FUNCTION_DECODING.add,
        memory_write: true,
        memory_read: MEMORY_READ.zero_extend,
        write_back: WRITE_BACK.memory_read,
    })
}

// Produce the ControlWord for an Instruction with opcode ST8
fn decode_st8(inst: Instruction) -> ControlWord {
    let source = get_field(inst, INSTRUCTION_SOURCE) as u32;
    let destination = get_field(inst, INSTRUCTION_DESTINATION) as u32;

    encode_control_word(ControlLines {
        condition: CONDITION.never,
        register_write: false,
        left: LEFT_SELECT.source,
        right: RIGHT_SELECT.immediate,
        source: source,
        destination: destination,
        immediate: IMMEDIATE.four,
        function: FUNCTION_DECODING.add,
        memory_write: true,
        memory_read: MEMORY_READ.zero_extend,
        write_back: WRITE_BACK.memory_read,
    })
}

// Produce the ControlWord for an Instruction with opcode LD8S
fn decode_ld8s(inst: Instruction) -> ControlWord {
    let source = get_field(inst, INSTRUCTION_SOURCE) as u32;
    let destination = get_field(inst, INSTRUCTION_DESTINATION) as u32;

    encode_control_word(ControlLines {
        condition: CONDITION.never,
        register_write: false,
        left: LEFT_SELECT.source,
        right: RIGHT_SELECT.immediate,
        source: source,
        destination: destination,
        immediate: IMMEDIATE.four,
        function: FUNCTION_DECODING.add,
        memory_write: true,
        memory_read: MEMORY_READ.sign_extend,
        write_back: WRITE_BACK.memory_read,
    })
}

// Produce the ControlWord for an Instruction with opcode JMP
fn decode_jmp(inst: Instruction) -> ControlWord {
    let jump_kind = get_field(inst, INSTRUCTION_JUMP_TYPE);

    let condition = match jump_kind {
        JUMP_TYPE_EQ => CONDITION.eq,
        JUMP_TYPE_NE => CONDITION.ne,
        JUMP_TYPE_LT => CONDITION.lt,
        JUMP_TYPE_GE => CONDITION.ge,
        JUMP_TYPE_LTS => CONDITION.lts,
        JUMP_TYPE_GES => CONDITION.ges,
        JUMP_TYPE_JR => CONDITION.always,
        JUMP_TYPE_JRAL => CONDITION.always,
        _ => unreachable!(),
    };

    let reg_write = match jump_kind {
        JUMP_TYPE_JRAL => true,
        _ => false,
    };

    encode_control_word(ControlLines {
        condition: condition,
        register_write: reg_write,
        left: LEFT_SELECT.program_counter,
        right: RIGHT_SELECT.immediate,
        source: CONTROL_LINK_REGISTER,
        destination: CONTROL_LINK_REGISTER,
        immediate: IMMEDIATE.nine as u32,
        function: FUNCTION_DECODING.add,
        memory_write: false,
        memory_read: MEMORY_READ.sign_extend,
        write_back: WRITE_BACK.program_counter,
    })
}

// Produce the ControlWord for an Instruction with opcode LUI
fn decode_lui(inst: Instruction) -> ControlWord {
    let destination = get_field(inst, INSTRUCTION_DESTINATION) as u32;

    encode_control_word(ControlLines {
        condition: CONDITION.never,
        register_write: true,
        left: LEFT_SELECT.source,
        right: RIGHT_SELECT.immediate,
        source: destination,
        destination: destination,
        immediate: IMMEDIATE.ten as u32,
        function: FUNCTION_DECODING.right,
        memory_write: false,
        memory_read: MEMORY_READ.sign_extend,
        write_back: WRITE_BACK.alu_result,
    })
}

// Produce the ControlWord for an Instruction with opcode AUIPC
fn decode_auipc(inst: Instruction) -> ControlWord {
    let destination = get_field(inst, INSTRUCTION_DESTINATION) as u32;

    encode_control_word(ControlLines {
        condition: CONDITION.never,
        register_write: true,
        left: LEFT_SELECT.program_counter,
        right: RIGHT_SELECT.immediate,
        source: destination,
        destination: destination,
        immediate: IMMEDIATE.ten as u32,
        function: FUNCTION_DECODING.add,
        memory_write: false,
        memory_read: MEMORY_READ.sign_extend,
        write_back: WRITE_BACK.alu_result,
    })
}
