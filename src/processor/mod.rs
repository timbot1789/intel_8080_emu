use std::fs;

#[derive(Debug)]
#[derive(Default)]
struct ConditionBits {
    carry: u8, // set if value is carried out of the highest order bit
    // aux_carry: u8, Not used for this project
    sign: u8, // set to 1 when bit 7 is set
    zero: u8, // set when result is equal to 0
    parity: u8 // set when result is even
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

    fn mov(&mut self, opcode: u8) {
        let reg_1: u8 = (opcode << 2) >> 5;
        let reg_2: u8 = opcode & 0b00000111;
        println!("mov {:x}, {:x}", reg_1, reg_2);
        let val = *self.get_register(reg_2);
        self.set_register(reg_1, val);
        self.pc += 1;
    }

    fn halt(&mut self) {
        println!("halt");
        self.halt = true;
        println!("memory location 0x2020: {:04x}", self.memory[0x2020]);
        println!("memory location 0x2121: {:04x}", self.memory[0x2121]);
        println!("memory location 0x1f1f: {:04x}", self.memory[0x1f1f]);

    }

    fn get_mem_addr(&mut self) -> u16 {
        let high_bits: u16 = (self.h as u16) << 8;
        let low_bits: u16 = self.l as u16;
        return high_bits | low_bits;
    }

    fn set_register(&mut self, reg: u8, value: u8) {
        *self.get_register(reg) = value;
    }

    fn inr(&mut self, opcode: u8) {
        let reg_code: u8 = opcode >> 3;
        println!("inr {:x}", reg_code);

        let register = self.get_register(reg_code);
        *register += 1;
        self.pc += 1;
    }

    fn inx(&mut self, opcode: u8) {
        let reg_pair = (opcode >> 4) & 0b1100;
        let mut pair_val = self.get_register_pair_value(reg_pair);
        pair_val += 1;
        let low_byte: u8 = (pair_val >> 8) as u8;
        let high_byte: u8 = (pair_val & 0xff) as u8;
        self.set_register_pair(reg_pair, low_byte, high_byte);
        self.pc += 1;
    }

    fn dcr(&mut self, opcode: u8) {
        let reg_code: u8 = opcode >> 3;

        let register = self.get_register(reg_code);
        *register -= 1;
        self.pc += 1;
    }

    fn dcx(&mut self, opcode: u8) {
        let reg_pair = (opcode >> 4) & 0b1100;
        let mut pair_val = self.get_register_pair_value(reg_pair);
        pair_val -= 1;
        let low_byte: u8 = (pair_val >> 8) as u8;
        let high_byte: u8 = (pair_val & 0xff) as u8;
        self.set_register_pair(reg_pair, low_byte, high_byte);
        self.pc += 1;
    }

    fn get_register(&mut self, reg: u8) -> &mut u8 {
        let mem_addr = self.get_mem_addr();
        
        return match reg {
            0 => &mut self.b,
            1 => &mut self.c,
            2 => &mut self.d,
            3 => &mut self.e,
            4 => &mut self.h,
            5 => &mut self.l,
            6 => &mut self.memory[mem_addr as usize],
            _ => &mut self.a,
        }
    }

    fn get_register_pair_value(&mut self, reg_pair: u8) -> u16{
        let mut high_byte: u16 = 0;
        let mut low_byte: u16 = 0;
        let mut sp_addr: u16 = 0;
        
        match reg_pair {
            0 => (|| {
                    low_byte = self.b as u16;
                    high_byte = self.c as u16;
                })(),
            1 => (|| {
                    low_byte = self.d as u16;
                    high_byte = self.e as u16;
                })(),
            2 => (|| {
                    low_byte = self.h as u16;
                    high_byte = self.l as u16;
                })(),
            3 => (|| {
                    sp_addr = self.sp;
                })(),
            _ => (),
        }

        return if reg_pair == 3 {
            sp_addr
        } else {
            (high_byte << 8) | low_byte
        };
    }


    fn set_register_pair(&mut self, reg_pair: u8, low_byte: u8, high_byte: u8) {

        match reg_pair {
            0 => (|| {
                    self.b = high_byte;
                    self.c = low_byte;
                })(),
            1 => (|| {
                    self.d = high_byte;
                    self.e = low_byte
                })(),
            2 => (|| {
                    self.h = high_byte;
                    self.l = low_byte;
                })(),
            3 => (|| {
                    let mut sp_addr : u16 = high_byte as u16;
                    sp_addr = sp_addr << 8;
                    sp_addr = sp_addr | low_byte as u16;
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
            0x02 | 0x12 => self.unimplemented_instruction(), // STAX
            0x03 | 0x13 | 0x23 | 0x33=> self.inx(opcode),
            0x04 | 0x0c |0x14 | 0x1c | 0x24 | 0x2c | 0x34 | 0x3c => self.inr(opcode),
            0x05 | 0x0d |0x15 | 0x1d | 0x25 | 0x2d | 0x35 | 0x3d => self.dcr(opcode),
            0x06 | 0x0e | 0x16 | 0x1e | 0x26 | 0x2e | 0x36 | 0x3e => self.mvi(opcode),
            0x07 => self.unimplemented_instruction(),
            0x09 |0x19 | 0x29 | 0x39 => self.unimplemented_instruction(), // DAD
            0x0a | 0x1a => self.unimplemented_instruction(), // LDAX
            0x0b | 0x1b | 0x2b | 0x3b => self.dcx(opcode),
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
            0x40..=0x75 |0x78..=0x7f => self.mov(opcode),
            0x76 => self.halt(),
            0x77 => self.unimplemented_instruction(),
            0x80..=0x87 => self.unimplemented_instruction(), // ADD
            0x88..=0x8f => self.unimplemented_instruction(), // ADC
            0x90..=0x97 => self.unimplemented_instruction(), // SUB
            0x98..=0x9f => self.unimplemented_instruction(), // SBB
            0xa0..=0xa7 => self.unimplemented_instruction(), // ANA
            0xa8..=0xaf => self.unimplemented_instruction(), // XRA
            0xb0..=0xb7 => self.unimplemented_instruction(), // ORA
            0xb8..=0xbf => self.unimplemented_instruction(), // CMP
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

