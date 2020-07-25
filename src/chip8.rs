use std::collections::HashMap;
const PROGRAM_START: usize = 0x200;

#[derive(Copy, Clone)]
struct Opcode(u16);

impl Opcode {

    fn x(self) -> usize {
        let Opcode(opcode) = self;
        opcode as usize & 0x0F00 >> 8
    }
    
    fn y(self) -> usize {
        let Opcode(opcode) = self;
        opcode as usize & 0x00F0 >> 4
    }
    
    fn kk(self) -> u8 {
        let Opcode(opcode) = self;
        opcode as u8 & 0x00FF
    }
    
    fn addr(self) -> usize {
        let Opcode(opcode) = self;
        opcode as usize & 0x0FFF
    }
    
}



pub struct Chip8 {
    memory: [u8; 4096], // 4KB
    v: [u8; 16], // Registers
    pc: usize, // Program Counter         
    sp: usize,  // Stack Pointer
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],

    gfx: [u8; 64*32],
}

impl Chip8 {
    pub fn new() -> Self {
        Chip8 {
            memory: [0 as u8; 4096],
            v: [0 as u8; 16],
            pc: PROGRAM_START,
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            stack: [0 as u16; 16],
            gfx: [0 as u8; 64*32],
        }
    }

    pub fn load_program(&mut self, path: &str) { unimplemented!("Chip8::load_program") }

    // Ok Result true if draw was called and screen should be updated, false otherwise
    pub fn cycle(&mut self) -> Result<bool, String> {
        let mut should_draw = false;
        let opcode = Opcode((self.memory[self.pc] as u16) << 8 | self.memory[self.pc + 1] as u16);

        // <OPCODE> - <DISASSEMBLY> - <DESCRIPTION>
        match opcode.0 & 0xF000 {
            0x0000 => {
                match opcode.0 & 0x000F {
                    // 00E0 - CLS - Clear Screen
                    0x0000 => for i in self.gfx.iter_mut() { *i = 0 },
                    // 00EE - RET - Return from Subroutine
                    0x000E => {
                        self.pc = self.stack[self.sp] as usize;
                        self.sp -= 1;
                    },
                    _ => return Err(format_err(opcode))
                }
            },
            // 1nnn - JP addr - Jump to location nnn
            0x1000 => {
                self.pc = opcode.addr();
            },
            // 2nnn - CALL addr - Call Subroutine at nnn
            0x2000 => {
                self.sp += 1;
                self.stack[self.sp] = self.pc as u16;
                self.pc = opcode.addr();
            },
            // 3xkk - SE Vx, byte - Skip next instruction if Vx == kk
            0x3000 => {
                let x = self.v[opcode.x()];
                let kk = opcode.kk();
                if x == kk {
                    self.pc += 2;
                }
                self.pc += 2;
            },
            // 4xkk - SNE Vx, byte - Skip next instruction if Vx != kk
            0x4000 => {
                let x = self.v[opcode.x()];
                let kk = opcode.kk();
                if x != kk {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            // 5xy0 - SE Vx, Vy - Skip next instruction if Vx == Vy
            0x5000 => {
                let x = self.v[opcode.x()];
                let y = self.v[opcode.y()];
                if x == y {
                    self.pc += 2;
                }
                self.pc += 2;
            },
            // 6xkk - LD Vx, byte - Set Vx = kk.
            0x6000 => {
                self.v[opcode.x()] = opcode.kk();
                self.pc += 2;
            }
            // 7xkk - ADD Vx, byte - Set Vx = Vx + kk.
            0x7000 => {
                let xindex = opcode.x();
                let x = self.v[xindex];
                self.v[xindex] = opcode.kk() + x;
                self.pc += 2;
            },       
            // 8---     
            0x8000 => {
                match opcode.0 & 0x000F {
                    // 8xy0 - LD Vx, Vy - Set Vx = Vy.
                    0x0000 => {
                        let xindex = opcode.x();
                        let yindex = opcode.y();
        
                        self.v[xindex] = self.v[yindex];
                        self.pc += 2;
                    },
                    // 8xy1 - OR Vx, Vy - Set Vx = Vx OR Vy.
                    0x0001 => {
                        let xindex = opcode.x();
                        let x = self.v[xindex];
                        let y = self.v[opcode.y()];

                        self.v[xindex] = x | y;

                        self.pc += 2;
                    },



                    _ => return Err(format_err(opcode))
                }
            },

            _ => return Err(format_err(opcode))
        }

        Ok(should_draw)
    }
}

fn format_err(Opcode(c): Opcode) -> String {
    format!("OPCODE: {} NOT VALID", c)
} 

