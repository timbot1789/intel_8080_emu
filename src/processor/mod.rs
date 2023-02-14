use std::fs;

#[derive(Debug)]
#[derive(Default)]
struct ConditionBits {
    carry: bool, // set if value is carried out of the highest order bit
    // aux_carry: u8, Not used for this project
    sign: bool, // set to 1 when bit 7 is set
    zero: bool, // set when result is equal to 0
    parity: bool // set when result is even
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

        while !self.halt {
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

    fn parity(&mut self, mut num: u16, size: usize) -> bool {
        let mut hamming_weight: u16 = 0;
        for _i in 0..size {
            hamming_weight += num & 0x1;
            num = num >> 1;
        }
        return (hamming_weight % 2) == 0;
    }

    fn get_mem_addr(&mut self) -> u16 {
        let high_bits: u16 = (self.h as u16) << 8;
        let low_bits: u16 = self.l as u16;
        return high_bits | low_bits;
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


    fn set_register(&mut self, reg: u8, value: u8) {
        *self.get_register(reg) = value;
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

    fn unimplemented_instruction(&mut self) {
        println!("Error: Unimplemented Instruction: {}\n", self.memory[self.pc as usize]);
    }

    fn lxi(&mut self, opcode: u8) {
        let reg_pair = opcode >> 4;
        println!("lxi {:x}, {:x}{:x}", reg_pair, self.memory[(self.pc) as usize], self.memory[(self.pc + 1) as usize]);
        self.set_register_pair(
            reg_pair, 
            self.memory[(self.pc + 1) as usize], 
            self.memory[(self.pc + 2) as usize]
        );
        self.pc += 2;
    }

    fn mvi(&mut self, opcode: u8) {
        let reg = opcode >> 3;
        println!("mvi {:x}, {:x}", reg, self.memory[(self.pc) as usize]);
        self.set_register(reg, self.memory[(self.pc) as usize]);
        self.pc += 1;
    }

    fn mov(&mut self, opcode: u8) {
        let reg_1: u8 = (opcode << 2) >> 5;
        let reg_2: u8 = opcode & 0b00000111;
        println!("mov {:x}, {:x}", reg_1, reg_2);
        let val = *self.get_register(reg_2);
        self.set_register(reg_1, val);
    }

    fn halt(&mut self) {
        println!("halt");
        self.halt = true;
        println!("memory location 0x2020: {:04x}", self.memory[0x2020]);
        println!("memory location 0x2121: {:04x}", self.memory[0x2121]);
        println!("memory location 0x1f1f: {:04x}", self.memory[0x1f1f]);
    }

    fn inr(&mut self, opcode: u8) {
        let reg_code: u8 = opcode >> 3;
        println!("inr {:x}", reg_code);

        let register = self.get_register(reg_code);
        let cur_val: u16 = (*register as u16) + 1;
        *register = (cur_val & 0x00ff) as u8;
        self.conditions.sign = (cur_val >> 7) != 0;
        self.conditions.zero = cur_val == 0;
        self.conditions.parity = self.parity(cur_val, 8);
        self.conditions.carry = (cur_val >> 8) > 0;
    }

    fn inx(&mut self, opcode: u8) {
        let reg_pair = (opcode >> 4) & 0b1100;
        let pair_val = self.get_register_pair_value(reg_pair) + 1;
        let low_byte: u8 = (pair_val >> 8) as u8;
        let high_byte: u8 = (pair_val & 0xff) as u8;
        self.set_register_pair(reg_pair, low_byte, high_byte);
    }

    fn dcr(&mut self, opcode: u8) {
        let reg_code: u8 = opcode >> 3;

        let register = self.get_register(reg_code);
        let cur_val: u16 = if *register > 0 {
            (*register as u16) - 1
        }
        else {
            0xff as u16
        };
        *register = (cur_val & 0x00ff) as u8;
        self.conditions.sign = (cur_val >> 7) != 0;
        self.conditions.zero = cur_val == 0;
        self.conditions.parity = self.parity(cur_val, 8);
        self.conditions.carry = cur_val > 0xff;
    }

    fn dcx(&mut self, opcode: u8) {
        let reg_pair = (opcode >> 4) & 0b1100;
        let mut pair_val = self.get_register_pair_value(reg_pair);
        pair_val -= 1;
        let low_byte: u8 = (pair_val >> 8) as u8;
        let high_byte: u8 = (pair_val & 0xff) as u8;
        self.set_register_pair(reg_pair, low_byte, high_byte);
    }

    fn add(&mut self, opcode: u8) {
        let reg_num: u8 = opcode & 0b111;
        let answer: u16 = (self.a as u16) + (*self.get_register(reg_num) as u16);
        self.conditions.sign = (answer & 0x80) != 0;
        self.conditions.zero = (answer & 0xff) == 0;
        self.conditions.parity = self.parity(answer & 0xff, 8);
        self.conditions.carry = answer > 0xff;
        self.a = (answer & 0xff) as u8;
    }

    fn adc(&mut self, opcode: u8) {
        let reg_num: u8 = opcode & 0b111;
        let answer: u16 = (self.a as u16) + (*self.get_register(reg_num) as u16) + (self.conditions.carry as u16);

        self.conditions.sign = (answer & 0x80) != 0;
        self.conditions.zero = (answer & 0xff) == 0;
        self.conditions.parity = self.parity(answer & 0xff, 8);
        self.conditions.carry = answer > 0xff;
        self.a = (answer & 0xff) as u8;
    }

    fn sub(&mut self, opcode: u8) {
        let reg_num: u8 = opcode & 0b111;
        let minuend = *self.get_register(reg_num);
        if self.a > minuend {
            self.a -= minuend;
            self.conditions.carry = true;
        } else {
            self.a = 0;
            self.conditions.carry = false;
        };
        self.conditions.sign = (self.a & 0x80) != 0;
        self.conditions.zero = self.a == 0;
        self.conditions.parity = self.parity(self.a as u16, 8);
    }

    fn sbb(&mut self, opcode: u8) {
        let reg_num: u8 = opcode & 0b111;
        let minuend = *self.get_register(reg_num);
        if self.a > (minuend + self.conditions.carry as u8) {
            self.a -= minuend + self.conditions.carry as u8;
            self.conditions.carry = true;
        } else {
            self.a = 0;
            self.conditions.carry = false;
        };
        self.conditions.sign = (self.a & 0x80) != 0;
        self.conditions.zero = self.a == 0;
        self.conditions.parity = self.parity(self.a as u16, 8);
    }
    
    fn ana(&mut self, opcode: u8) {
        let reg_num: u8 = opcode & 0b111;
        self.a = self.a & *self.get_register(reg_num);
        self.conditions.carry = false;
        self.conditions.sign = (self.a & 0x80) != 0;
        self.conditions.zero = self.a == 0;
        self.conditions.parity = self.parity(self.a as u16, 8);
    }

    fn xra(&mut self, opcode: u8) {
        let reg_num: u8 = opcode & 0b111;
        self.a = self.a ^ *self.get_register(reg_num);
        self.conditions.carry = false;
        self.conditions.sign = (self.a & 0x80) != 0;
        self.conditions.zero = self.a == 0;
        self.conditions.parity = self.parity(self.a as u16, 8);
    }

    fn ora(&mut self, opcode: u8) {
        let reg_num: u8 = opcode & 0b111;
        self.a = self.a | *self.get_register(reg_num);
        self.conditions.carry = false;
        self.conditions.sign = (self.a & 0x80) != 0;
        self.conditions.zero = self.a == 0;
        self.conditions.parity = self.parity(self.a as u16, 8);
    }

    fn cmp(&mut self, opcode: u8) {
        let reg_num: u8 = opcode & 0b111;
        let minuend = *self.get_register(reg_num);
        let mut acc = self.a;
        if acc > minuend {
            acc -= minuend;
            self.conditions.carry = true;
        } else {
            acc = 0;
            self.conditions.carry = false;
        };
        self.conditions.sign = (acc & 0x80) != 0;
        self.conditions.zero = acc == 0;
        self.conditions.parity = self.parity(acc as u16, 8);
    }

    fn pchl(&mut self) { // Set program counter to address in HL registers
        let high_bits: u16 = (self.h as u16)<< 8;
        let low_bits: u16 = self.l as u16;
        self.pc = high_bits | low_bits;
    }

    fn jmp(&mut self) {
        let pc = self.pc as usize;
        let low_byte: u16 = self.memory[pc + 1] as u16;
        let high_byte: u16 = (self.memory[pc + 2] as u16) << 8 ;
        let addr = high_byte | low_byte;

        self.pc = addr;
    }

    fn jump(&mut self, opcode: u8) {

        let cmp_val = match (opcode >> 3) & 0b00111 {
            0 => { !self.conditions.zero }, // JNZ
            1 => { self.conditions.zero }, // JZ
            2 => { !self.conditions.carry }, // JNC
            3 => { self.conditions.carry }, // JC
            4 => { !self.conditions.parity }, // JPO
            5 => { self.conditions.parity }, // JPE
            6 => { !self.conditions.sign }, // JP
            7 => { self.conditions.sign }, // JM
            _ => { false }
        };

        if cmp_val { self.jmp() }
    }

    fn call(&mut self) {
        let sp: usize = self.sp as usize;
        let ret: u16 = self.pc + 2;
        let low_byte: u8 = (ret & 0xff00) as u8;
        let high_byte: u8 = (ret >> 8) as u8 ;
        self.memory[sp - 1] = high_byte;
        self.memory[sp - 2] = low_byte;
        self.sp -= 2;
        self.jmp();
    }

    fn ret(&mut self) {
        let sp: usize = self.sp as usize;
        let low_byte: u16 = self.memory[sp] as u16;
        let high_byte: u16 = (self.memory[sp + 1] as u16) << 8 ;
        self.sp += 2;
        self.pc = high_byte | low_byte;
    }

    fn run_one_command(&mut self) {
        let opcode: u8 = self.memory[self.pc as usize];
        self.pc += 1;
        return match opcode {
            0x00 => (|| {println!("NOP"); })(),
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
            0x80..=0x87 => self.add(opcode), // ADD
            0x88..=0x8f => self.adc(opcode), // ADC
            0x90..=0x97 => self.sub(opcode), // SUB
            0x98..=0x9f => self.sbb(opcode), // SBB
            0xa0..=0xa7 => self.ana(opcode), // ANA
            0xa8..=0xaf => self.xra(opcode), // XRA
            0xb0..=0xb7 => self.ora(opcode), // ORA
            0xb8..=0xbf => self.cmp(opcode), // CMP
            0xc2 | 0xca | 0xd2 | 0xda | 0xe2 | 0xea => self.jump(opcode),
            0xc3 => self.jmp(),
            0xc0 => self.unimplemented_instruction(),
            0xc1 => self.unimplemented_instruction(),
            0xd1 => self.unimplemented_instruction(),
            0xe1 => self.unimplemented_instruction(),
            0xf1 => self.unimplemented_instruction(),
            0xc4 => self.unimplemented_instruction(),
            0xc5 => self.unimplemented_instruction(),
            0xd5 => self.unimplemented_instruction(),
            0xe5 => self.unimplemented_instruction(),
            0xf5 => self.unimplemented_instruction(),
            0xc6 => self.unimplemented_instruction(),
            0xc7 => self.unimplemented_instruction(),
            0xc8 => self.unimplemented_instruction(),
            0xc9 => self.ret(),
            0xcc => self.unimplemented_instruction(),
            0xcd => self.call(),
            0xce => self.unimplemented_instruction(),
            0xcf => self.unimplemented_instruction(),
            0xd0 => self.unimplemented_instruction(),
            0xd3 => self.unimplemented_instruction(),
            0xd4 => self.unimplemented_instruction(),
            0xd6 => self.unimplemented_instruction(),
            0xd7 => self.unimplemented_instruction(),
            0xd8 => self.unimplemented_instruction(),
            0xdb => self.unimplemented_instruction(),
            0xdc => self.unimplemented_instruction(),
            0xde => self.unimplemented_instruction(),
            0xdf => self.unimplemented_instruction(),
            0xe0 => self.unimplemented_instruction(),
            0xe3 => self.unimplemented_instruction(),
            0xe4 => self.unimplemented_instruction(),
            0xe6 => self.unimplemented_instruction(),
            0xe7 => self.unimplemented_instruction(),
            0xe8 => self.unimplemented_instruction(),
            0xe9 => self.pchl(),
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

