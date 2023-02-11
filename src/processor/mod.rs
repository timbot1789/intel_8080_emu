use std::fs;

#[derive(Debug)]
#[derive(Default)]
struct ConditionBits {
    carry: u8,
    aux_carry: u8,
    sign: u8,
    zero: u8,
    parity: u8
}

#[derive(Debug)]
#[derive(Default)]
pub struct Processor {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
    conditions: ConditionBits,
    halt: bool,
    memory: Vec<u8>,
}

pub fn make_processor() -> Processor {
    return Processor { ..Default::default()};
}

impl Processor {

    pub fn run_program(&mut self, path: &str) -> String{

        self.initialize_memory(path);

        while (self.pc < self.memory.len() as u16) && !self.halt {
            self.run_one_command();
        }

        return format!("Final Processor State:\n{:#?}", self);
    }

    fn initialize_memory(&mut self, path: &str) {
        self.memory.extend_from_slice(&fs::read(path)
        .expect("Should have been able to read the file"));
        println!("{:#?}", self.memory);
        self.memory.resize_with(0xffff, || {0});
    }

    fn unimplemented_instruction(&mut self) {
        println!("Error: Unimplemented Instruction: {}\n", self.memory[self.pc as usize]);
        self.pc += 1;
    }

    fn lxi(&mut self, opcode: u8) {
        let reg_pair = opcode >> 4;
        println!("lxi {:x}, {:x}{:x}", reg_pair, self.memory[(self.pc + 1) as usize], self.memory[(self.pc + 2) as usize]);
        self.set_register_pair(
            reg_pair, 
            self.memory[(self.pc + 1) as usize], 
            self.memory[(self.pc + 2) as usize]
        );
        self.pc += 3;
    }

    fn mvi(&mut self, opcode: u8) {
        let reg = opcode >> 3;
        println!("mvi {:x}, {:x}", reg, self.memory[(self.pc + 1) as usize]);
        self.set_register(reg, self.memory[(self.pc + 1) as usize]);
        self.pc += 2;
    }

    fn get_reg_val(&mut self, reg: u8) -> u8 {
        return match reg {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            6 => self.load_from_memory(),
            7 => self.a,
            _ => 0,
        }
    }

    fn mov(&mut self, opcode: u8) {
        let reg_1: u8 = (opcode << 2) >> 5;
        let reg_2: u8 = opcode & 0b00000111;
        println!("mov {:x}, {:x}", reg_1, reg_2);
        let val = self.get_reg_val(reg_2);
        self.set_register(reg_1, val);
        self.pc += 1;
    }

    fn halt(&mut self) {
        println!("halt");
        self.halt = true;
    }

    fn get_mem_addr(&mut self) -> u16 {
        let high_bits: u16 = (self.h as u16) << 8;
        let low_bits: u16 = self.l as u16;
        return high_bits | low_bits;
    }

    fn store_to_memory(&mut self, value: u8) -> (){
        let addr = self.get_mem_addr();
        self.memory[addr as usize] = value;
    }

    fn load_from_memory(&mut self) -> u8 {
        let addr = self.get_mem_addr();
        return self.memory[addr as usize];
    }

    fn set_register(&mut self, reg: u8, value: u8) {
        match reg {
            0 => self.b = value,
            1 => self.c = value,
            2 => self.d = value,
            3 => self.e = value,
            4 => self.h = value,
            5 => self.l = value,
            6 => self.store_to_memory(value),
            7 => self.a = value,
            _ => (),
        }
    }

    fn set_register_pair(&mut self, reg_pair: u8, first_byte: u8, second_byte: u8) {

        match reg_pair {
            0 => (|| {
                    self.b = second_byte;
                    self.c = first_byte;
                })(),
            1 => (|| {
                    self.d = second_byte;
                    self.e = first_byte
                })(),
            2 => (|| {
                    self.h = second_byte;
                    self.l = first_byte;
                })(),
            3 => (|| {
                    let mut sp_addr : u16 = second_byte as u16;
                    sp_addr = sp_addr << 8;
                    sp_addr = sp_addr | first_byte as u16;
                    self.sp = sp_addr
                })(),
            _ => (),
        }
    }

    fn run_one_command(&mut self) {
        let opcode: u8 = self.memory[self.pc as usize];
        return match opcode {
            0x00 => (|| {println!("NOP"); self.pc += 1})(),
            0x01 | 0x11 | 0x21 | 0x31 => self.lxi(opcode),
            0x06 | 0x0e | 0x16 | 0x1e | 0x26 | 0x2e | 0x36 | 0x3e => self.mvi(opcode),
            0x12 => self.unimplemented_instruction(),
            0x03 => self.unimplemented_instruction(),
            0x13 => self.unimplemented_instruction(),
            0x23 => self.unimplemented_instruction(),
            0x33 => self.unimplemented_instruction(),
            0x04 => self.unimplemented_instruction(),
            0x0c => self.unimplemented_instruction(),
            0x14 => self.unimplemented_instruction(),
            0x1c => self.unimplemented_instruction(),
            0x24 => self.unimplemented_instruction(),
            0x2c => self.unimplemented_instruction(),
            0x34 => self.unimplemented_instruction(),
            0x3c => self.unimplemented_instruction(),
            0x05 => self.unimplemented_instruction(),
            0x0d => self.unimplemented_instruction(),
            0x15 => self.unimplemented_instruction(),
            0x17 => self.unimplemented_instruction(),
            0x1d => self.unimplemented_instruction(),
            0x25 => self.unimplemented_instruction(),
            0x2d => self.unimplemented_instruction(),
            0x3d => self.unimplemented_instruction(),
            0x35 => self.unimplemented_instruction(),
            0x07 => self.unimplemented_instruction(),
            0x09 => self.unimplemented_instruction(),
            0x19 => self.unimplemented_instruction(),
            0x29 => self.unimplemented_instruction(),
            0x39 => self.unimplemented_instruction(),
            0x0a => self.unimplemented_instruction(),
            0x1a => self.unimplemented_instruction(),
            0x0b => self.unimplemented_instruction(),
            0x1b => self.unimplemented_instruction(),
            0x2b => self.unimplemented_instruction(),
            0x3b => self.unimplemented_instruction(),
            0x0f => self.unimplemented_instruction(),
            0x1f => self.unimplemented_instruction(),
            0x22 => self.unimplemented_instruction(),
            0x27 => self.unimplemented_instruction(),
            0x2a => self.unimplemented_instruction(),
            0x2f => self.unimplemented_instruction(),
            0x32 => self.unimplemented_instruction(),
            0x37 => self.unimplemented_instruction(),
            0x3a => self.unimplemented_instruction(),
            0x3f => self.unimplemented_instruction(),
            0x40..=0x75 => self.mov(opcode),
            0x76 => self.halt(),
            0x77 => self.unimplemented_instruction(),
            0x78..=0x7f => self.mov(opcode),
            0x80..=0x87 => self.unimplemented_instruction(),
            0x88..=0x8f => self.unimplemented_instruction(),
            0x90..=0x97 => self.unimplemented_instruction(),
            0x98..=0x9f => self.unimplemented_instruction(),
            0xa0..=0xa7 => self.unimplemented_instruction(),
            0xa8..=0xaf => self.unimplemented_instruction(),
            0xb0..=0xb7 => self.unimplemented_instruction(),
            0xb8..=0xbf => self.unimplemented_instruction(),
            0xc0 => self.unimplemented_instruction(),
            0xc1 => self.unimplemented_instruction(),
            0xd1 => self.unimplemented_instruction(),
            0xe1 => self.unimplemented_instruction(),
            0xf1 => self.unimplemented_instruction(),
            0xc2 => self.unimplemented_instruction(),
            0xc3 => self.unimplemented_instruction(),
            0xc4 => self.unimplemented_instruction(),
            0xc5 => self.unimplemented_instruction(),
            0xd5 => self.unimplemented_instruction(),
            0xe5 => self.unimplemented_instruction(),
            0xf5 => self.unimplemented_instruction(),
            0xc6 => self.unimplemented_instruction(),
            0xc7 => self.unimplemented_instruction(),
            0xc8 => self.unimplemented_instruction(),
            0xc9 => self.unimplemented_instruction(),
            0xca => self.unimplemented_instruction(),
            0xcc => self.unimplemented_instruction(),
            0xcd => self.unimplemented_instruction(),
            0xce => self.unimplemented_instruction(),
            0xcf => self.unimplemented_instruction(),
            0xd0 => self.unimplemented_instruction(),
            0xd2 => self.unimplemented_instruction(),
            0xd3 => self.unimplemented_instruction(),
            0xd4 => self.unimplemented_instruction(),
            0xd6 => self.unimplemented_instruction(),
            0xd7 => self.unimplemented_instruction(),
            0xd8 => self.unimplemented_instruction(),
            0xda => self.unimplemented_instruction(),
            0xdb => self.unimplemented_instruction(),
            0xdc => self.unimplemented_instruction(),
            0xde => self.unimplemented_instruction(),
            0xdf => self.unimplemented_instruction(),
            0xe0 => self.unimplemented_instruction(),
            0xe2 => self.unimplemented_instruction(),
            0xe3 => self.unimplemented_instruction(),
            0xe4 => self.unimplemented_instruction(),
            0xe6 => self.unimplemented_instruction(),
            0xe7 => self.unimplemented_instruction(),
            0xe8 => self.unimplemented_instruction(),
            0xe9 => self.unimplemented_instruction(),
            0xea => self.unimplemented_instruction(),
            0xeb => self.unimplemented_instruction(),
            0xec => self.unimplemented_instruction(),
            0xee => self.unimplemented_instruction(),
            0xef => self.unimplemented_instruction(),
            0xf0 => self.unimplemented_instruction(),
            0xf2 => self.unimplemented_instruction(),
            0xf3 => self.unimplemented_instruction(),
            0xf4 => self.unimplemented_instruction(),
            0xf6 => self.unimplemented_instruction(),
            0xf7 => self.unimplemented_instruction(),
            0xf8 => self.unimplemented_instruction(),
            0xf9 => self.unimplemented_instruction(),
            0xfa => self.unimplemented_instruction(),
            0xfb => self.unimplemented_instruction(),
            0xfc => self.unimplemented_instruction(),
            0xfe => self.unimplemented_instruction(),
            0xff => self.unimplemented_instruction(),
            _ => self.unimplemented_instruction(),
        }
    }
}

