use crate::util::Flat2DArray;
use rand::{thread_rng, Rng};
use std::num::Wrapping;


const PROGRAM_START: u16 = 0x200;

#[derive(Copy, Clone)]
struct Opcode(u16);

impl Opcode {

    fn x(self) -> usize {
        ((self.0 & 0x0F00) >> 8) as usize
    }
    
    fn y(self) -> usize {
        ((self.0 & 0x00F0) >> 4) as usize
    }
    
    fn kk(self) -> u8 {
        (self.0 & 0x00FF) as u8
    }
    
    fn addr(self) -> u16 {
        self.0 & 0x0FFF
    }
    
}

#[derive(Copy, Clone, Default)]
#[repr(C)]
struct Pixel {
    r: u8, g: u8, b: u8, 
    _pad: u8 // ignored
}

impl Pixel {
    fn new(r: u8, g: u8, b: u8) -> Self {
        Pixel {
            r, g, b,
            _pad: 0 // ignoSred
        }
    }

    fn white() -> Self {
        Pixel::new(255, 255, 255)
    }
}

const FONT_MEM_OFFSET: u16 = 0x00;
const CHIP8_FONTSET: [u8; 80] = [ 
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
	0x20, 0x60, 0x20, 0x20, 0x70, // 1
	0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
	0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
	0x90, 0x90, 0xF0, 0x10, 0x10, // 4
	0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
	0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
	0xF0, 0x10, 0x20, 0x40, 0x40, // 7
	0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
	0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
	0xF0, 0x90, 0xF0, 0x90, 0x90, // A
	0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
	0xF0, 0x80, 0x80, 0x80, 0xF0, // C
	0xE0, 0x90, 0x90, 0x90, 0xE0, // D
	0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
	0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Chip8 {
    memory: [u8; 4096], // 4KB
    v: [u8; 16], // Registers
    i: u16,
    pc: u16, // Program Counter         
    sp: u16,  // Stack Pointer
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    keyboard: [bool; 16],

    gfx: Flat2DArray<u8>,
}

impl Chip8 {

    pub const DISPLAY_W: u32 = 64;
    pub const DISPLAY_H: u32 = 32;

    const DISPLAY_WH: usize = Chip8::DISPLAY_W as usize * Chip8::DISPLAY_H as usize;

    pub fn new() -> Self {
        let mut c = Chip8 {
            memory: [0 as u8; 4096],
            v: [0 as u8; 16],
            i: 0,
            pc: PROGRAM_START,
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            stack: [0 as u16; 16],
            keyboard: [false; 16],
            gfx: Flat2DArray::new(Chip8::DISPLAY_W as usize, Chip8::DISPLAY_H as usize),
        };
        // load fontset
        for i in 0..CHIP8_FONTSET.len() {
            c.memory[i] = CHIP8_FONTSET[i];
        }
        // unsafe {
        //     let len = CHIP8_FONTSET.len();
        //     let src = CHIP8_FONTSET.as_ptr();
        //     let dst = c.memory.as_mut_ptr().offset(FONT_MEM_OFFSET as isize);

        //     std::ptr::copy_nonoverlapping(src, dst, len);
        // }

        c
    }

    pub fn render_to_pixels(&self) -> Vec<u8> {

        let mut buff = vec![Pixel::default(); Chip8::DISPLAY_WH];

        for (i, _) in self.gfx.data.iter().enumerate() {
            if self.gfx.data[i] == 0xFF {
                buff[i] = Pixel::white();
            } else {
                buff[i] = Pixel::default();
            }
        }

        unsafe { std::mem::transmute(buff) }
    }

    /*
        NOTE: the chip8 key symbols are also the hexidecimal position they are in 
        in our keyboard buffer on Chip8 struct

        Keyboard       Chip8 Keypad     
        +-+-+-+-+      +-+-+-+-+        
        |1|2|3|4|      |1|2|3|C|        
        +-+-+-+-+      +-+-+-+-+        
        |Q|W|E|R|      |4|5|6|D|        
        +-+-+-+-+  =>  +-+-+-+-+    
        |A|S|D|F|      |7|8|9|E|        
        +-+-+-+-+      +-+-+-+-+        
        |Z|X|C|V|      |A|0|B|F|        
        +-+-+-+-+      +-+-+-+-+        
    */
    pub fn process_input(&mut self, event: &sdl2::event::Event) {
        use sdl2::event::Event;
        use sdl2::keyboard::Keycode;
        match event {
            Event::KeyDown { keycode: Some(Keycode::Num1), .. } => self.keyboard[0x1] = true,
            Event::KeyDown { keycode: Some(Keycode::Num2), .. } => self.keyboard[0x2] = true,
            Event::KeyDown { keycode: Some(Keycode::Num3), .. } => self.keyboard[0x3] = true,
            Event::KeyDown { keycode: Some(Keycode::Num4), .. } => self.keyboard[0xC] = true,
            Event::KeyDown { keycode: Some(Keycode::Q),    .. } => self.keyboard[0x4] = true,
            Event::KeyDown { keycode: Some(Keycode::W),    .. } => self.keyboard[0x5] = true,
            Event::KeyDown { keycode: Some(Keycode::E),    .. } => self.keyboard[0x6] = true,
            Event::KeyDown { keycode: Some(Keycode::R),    .. } => self.keyboard[0xD] = true,
            Event::KeyDown { keycode: Some(Keycode::A),    .. } => self.keyboard[0x7] = true,
            Event::KeyDown { keycode: Some(Keycode::S),    .. } => self.keyboard[0x8] = true,
            Event::KeyDown { keycode: Some(Keycode::D),    .. } => self.keyboard[0x9] = true,
            Event::KeyDown { keycode: Some(Keycode::F),    .. } => self.keyboard[0xE] = true,
            Event::KeyDown { keycode: Some(Keycode::Z),    .. } => self.keyboard[0xA] = true,
            Event::KeyDown { keycode: Some(Keycode::X),    .. } => self.keyboard[0x0] = true,
            Event::KeyDown { keycode: Some(Keycode::C),    .. } => self.keyboard[0xB] = true,
            Event::KeyDown { keycode: Some(Keycode::V),    .. } => self.keyboard[0xF] = true,


            Event::KeyUp { keycode: Some(Keycode::Num1), .. } => self.keyboard[0x1] = false,
            Event::KeyUp { keycode: Some(Keycode::Num2), .. } => self.keyboard[0x2] = false,
            Event::KeyUp { keycode: Some(Keycode::Num3), .. } => self.keyboard[0x3] = false,
            Event::KeyUp { keycode: Some(Keycode::Num4), .. } => self.keyboard[0xC] = false,
            Event::KeyUp { keycode: Some(Keycode::Q),    .. } => self.keyboard[0x4] = false,
            Event::KeyUp { keycode: Some(Keycode::W),    .. } => self.keyboard[0x5] = false,
            Event::KeyUp { keycode: Some(Keycode::E),    .. } => self.keyboard[0x6] = false,
            Event::KeyUp { keycode: Some(Keycode::R),    .. } => self.keyboard[0xD] = false,
            Event::KeyUp { keycode: Some(Keycode::A),    .. } => self.keyboard[0x7] = false,
            Event::KeyUp { keycode: Some(Keycode::S),    .. } => self.keyboard[0x8] = false,
            Event::KeyUp { keycode: Some(Keycode::D),    .. } => self.keyboard[0x9] = false,
            Event::KeyUp { keycode: Some(Keycode::F),    .. } => self.keyboard[0xE] = false,
            Event::KeyUp { keycode: Some(Keycode::Z),    .. } => self.keyboard[0xA] = false,
            Event::KeyUp { keycode: Some(Keycode::X),    .. } => self.keyboard[0x0] = false,
            Event::KeyUp { keycode: Some(Keycode::C),    .. } => self.keyboard[0xB] = false,
            Event::KeyUp { keycode: Some(Keycode::V),    .. } => self.keyboard[0xF] = false,
            _ => {}
        }
    }

    pub fn load_program(&mut self, path: &str) -> Result<(), std::io::Error> { 
        
        let buffer = std::fs::read(path)?;
        for i in 0..buffer.len() {
            self.memory[PROGRAM_START as usize + i] = buffer[i];
        }
        Ok(())
    }

    // Ok Result true if draw was called and screen should be updated, false otherwise
    pub fn cycle(&mut self) -> Result<bool, String> {
        let mut should_draw = false;
        let opcode = { 
            let pc = self.pc as usize;
            Opcode((self.memory[pc] as u16) << 8 | self.memory[pc + 1] as u16)
        };
        self.pc += 2;
        // <OPCODE> - <DISASSEMBLY> - <DESCRIPTION>
        match opcode.0 & 0xF000 {
            0x0000 => {
                match opcode.0 & 0x000F {
                    // 00E0 - CLS - Clear Screen
                    0x0000 => { 
                        self.gfx.clear();
                        should_draw = true;
                        
                    },
                    // 00EE - RET - Return from Subroutine
                    0x000E => {
                        self.pc = self.stack[self.sp as usize];
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
                self.stack[self.sp as usize] = self.pc;
                self.pc = opcode.addr();
            },
            // 3xkk - SE Vx, byte - Skip next instruction if Vx == kk
            0x3000 => {
                let x = self.v[opcode.x()];
                let kk = opcode.kk();
                if x == kk {
                    self.pc += 2;
                }
                
            },
            // 4xkk - SNE Vx, byte - Skip next instruction if Vx != kk
            0x4000 => {
                let x = self.v[opcode.x()];
                let kk = opcode.kk();
                if x != kk {
                    self.pc += 2;
                }
                
            }
            // 5xy0 - SE Vx, Vy - Skip next instruction if Vx == Vy
            0x5000 => {
                let x = self.v[opcode.x()];
                let y = self.v[opcode.y()];
                if x == y {
                    self.pc += 2;
                }
                
            },
            // 6xkk - LD Vx, byte - Set Vx = kk.
            0x6000 => {
                self.v[opcode.x()] = opcode.kk();
                
            }
            // 7xkk - ADD Vx, byte - Set Vx = Vx + kk.
            0x7000 => {
                let vx = opcode.x();
                let x = Wrapping(self.v[vx]);
                let byte = Wrapping(opcode.kk());

                self.v[vx] = (x + byte).0;                
            },       
            // 8---     
            0x8000 => {
                match opcode.0 & 0x000F {
                    // 8xy0 - LD Vx, Vy - Set Vx = Vy.
                    0x0000 => {
                        let xindex = opcode.x();
                        let yindex = opcode.y();
        
                        self.v[xindex] = self.v[yindex];
                        
                    },
                    // 8xy1 - OR Vx, Vy - Set Vx = Vx OR Vy.
                    0x0001 => {
                        let xindex = opcode.x();
                        let y = self.v[opcode.y()];

                        self.v[xindex] |= y;

                        
                    },
                    // 8xy2 - AND Vx, Vy - Set Vx = Vx AND Vy.
                    0x0002 => {
                        let xindex = opcode.x();
                        let y = self.v[opcode.y()];

                        self.v[xindex] &= y;

                        
                    },
                    // 8xy3 - XOR Vx, Vy - Set Vx = Vx XOR Vy.
                    0x0003 => {
                        let vx = opcode.x();
                        let vy = opcode.y();
                        self.v[vx] ^= self.v[vy];                        
                    }
                    // 8xy4 - ADD Vx, Vy - Set Vx = Vx + Vy, set VF = carry.
                    0x0004 => {
                        let xindex = opcode.x();
                        let x = self.v[xindex] as u32;
                        let y = self.v[opcode.y()] as u32;

                        let result = x + y;

                        self.v[xindex] = (result & 0xFF) as u8;

                        // Check for overflow, set carry flag if we did
                        self.v[0xF] = if result > 255 { 1 } else { 0 };
                        
                    },
                    // 8xy5 - SUB Vx, Vy - Set Vx = Vx - Vy, set VF = NOT borrow.
                    0x0005 => {                        
                        let xindex = opcode.x();
                        let x = Wrapping(self.v[xindex]);
                        let y = Wrapping(self.v[opcode.y()]);

                        self.v[0xF] = if x > y { 1 } else { 0 };
                        self.v[xindex] = (x - y).0;

                        
                    },
                    // 8xy6 - SHR Vx {, Vy} - Set Vx = Vx SHR 1 (Shift Right)
                    0x0006 => {
                        let xindex = opcode.x();
                        let x = self.v[xindex];

                        // set VF to 1 if least significant bit of x is 1. otherwise 0
                        self.v[0xF] = x & 1;

                        self.v[xindex] = x >> 1;

                        
                    },
                    // 8xy7 - SUBN Vx, Vy - Set Vx = Vy - Vx, set VF = NOT borrow.
                    0x0007 => {
                        let xindex = opcode.x();
                        let x = Wrapping(self.v[xindex]);
                        let y = Wrapping(self.v[opcode.y()]);

                        self.v[0xF] = if y > x { 1 } else { 0 };
                        self.v[xindex] = (y - x).0;

                        
                    },
                    // 8xyE - SHL Vx {, Vy} - Set Vx = Vx SHL 1. (Shift Left)
                    0x000E => {
                        let xindex = opcode.x();
                        let x = self.v[xindex];

                        // set VF to 1 if most significant bit of x is 1. otherwise 0
                        self.v[0xF] = (x & 0x80) >> 7;

                        self.v[xindex] = x << 1;

                        
                    }

                    _ => return Err(format_err(opcode))
                }
            },
            // 9xy0 - SNE Vx, Vy - Skip next instruction if Vx != Vy.
            0x9000 => {
               let x = self.v[opcode.x()];
               let y = self.v[opcode.y()];
               if x != y {
                   self.pc += 2;
               } 
               
            },
            // Annn - LD I, addr - Set I = nnn.
            0xA000 => {
                self.i = opcode.addr();
                
            },
            // Bnnn - JP V0, addr - Jump to location nnn + V0.
            0xB000 => {
                self.pc = opcode.addr() + self.v[0] as u16;
            },
            // Cxkk - RND Vx, byte - Set Vx = random byte AND kk.
            0xC000 => {
                self.v[opcode.x()] = random_8bit() & opcode.kk();
                
            }
            // Dxyn - DRW Vx, Vy, nibble - Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
            0xD000 => {
                let x = self.v[opcode.x()];
                let y = self.v[opcode.y()];
                let height = opcode.0 & 0x000F;
                self.v[0xF] = 0;

                // Each row in sprite is byte. Each pixel in sprite is a Bit. 
                // Example: Sprite of the number 0
                // from: http://www.multigesture.net/articles/how-to-write-an-emulator-chip-8-interpreter/
                /*
                 *  DEC   HEX    BIN         RESULT
                 *  ---------------------------------
                 *  240   0xF0   1111 0000    ****
                 *  144   0x90   1001 0000    *  *
                 *  144   0x90   1001 0000    *  *
                 *  144   0x90   1001 0000    *  *
                 *  240   0xF0   1111 0000    ****
                 */
                for row in 0..height as usize {

                    let sprite_byte = self.memory[self.i as usize + row];
                    for col in 0..8 {

                        let xoffset = (x as usize + col) % Chip8::DISPLAY_W as usize;
                        let yoffset = (y as usize + row) % Chip8::DISPLAY_H as usize;
                        
                        let sprite_bit = sprite_byte & (0x80 >> col);

                        let gfx_byte = *self.gfx.get(xoffset, yoffset);
                        if sprite_bit != 0 {
                            if gfx_byte == 0xFF {
                                self.v[0xF] = 1;
                            }
                            self.gfx.set(xoffset, yoffset, gfx_byte ^ 0xFF);
                        }
                        
                    }
                }

                should_draw = true;
                
            },
            // E000
            0xE000 => {
                match opcode.0 & 0x00FF {
                    // Ex9E - SKP Vx - Skip next instruction if key with the value of Vx is pressed.
                    0x009E => {
                        let x = self.v[opcode.x()];
                        if self.keyboard[x as usize] {
                            self.pc += 2;
                        }
                        
                    },
                    // ExA1 - SKNP Vx - Skip next instruction if key with the value of Vx is not pressed.
                    0x00A1 => {
                        let x = self.v[opcode.x()];
                        if !self.keyboard[x as usize] {
                            self.pc += 2;
                        }
                        
                    }
                    _ => return Err(format_err(opcode))
                }
            },
            // F000
            0xF000 => {
                match opcode.0 & 0x00FF {
                    // Fx07 - LD Vx, DT - Set Vx = delay timer value.
                    0x0007 => {
                        self.v[opcode.x()] = self.delay_timer;                        
                    },
                    // Fx0A - LD Vx, K - Wait for a key press, store the value of the key in Vx.
                    0x000A => {
                        for (i, state) in self.keyboard.iter().enumerate() {
                            if *state {
                                self.v[opcode.x()] = i as u8;
                                // Keep decrementing program counter by 2 to 
                                // simulate 'waiting' for a keypress. We just run this instruction over and over until we have a key press
                                // from: https://austinmorlan.com/posts/chip8_emulator/
                                self.pc -= 2;
                                break;
                            }
                        }
                    },
                    // Fx15 - LD DT, Vx - Set delay timer = Vx.
                    0x0015 => {
                        self.delay_timer = self.v[opcode.x()];                        
                    }
                    // Fx18 - LD ST, Vx - Set sound timer = Vx.
                    0x0018 => {
                        self.sound_timer = self.v[opcode.x()];
                    },
                    // Fx1E - ADD I, Vx - Set I = I + Vx.
                    0x001E => {
                        self.i = self.i + self.v[opcode.x()] as u16;
                    },
                    // Fx29 - LD F, Vx - Set I = location of sprite for digit Vx.
                    0x0029 => {
                        let x = self.v[opcode.x()] as u16;
                        // Each font is 5 bytes long in memory, so we take the 
                        // given value x from register and multiply it by 5 to get the position of the font requested
                        self.i = FONT_MEM_OFFSET + (x * 5);
                    },
                    // Fx33 - LD B, Vx - Store BCD representation of Vx in memory locations I, I+1, and I+2.
                    0x0033 => {
                        // The interpreter takes the decimal value of Vx, 
                        // and places the hundreds digit in memory at location in I,
                        // the tens digit at location I+1, and the ones digit at location I+2.

                        let mut x = self.v[opcode.x()];
                        let i = self.i as usize;
                        // Ones
                        self.memory[i + 2] = x % 10;
                        x /= 10;

                        // Tens
                        self.memory[i + 1] = x % 10;
                        x /= 10;

                        // Hundreds
                        self.memory[i] = x % 10;
                    },
                    // Fx55 - LD [I], Vx - Store registers V0 through Vx in memory starting at location I.
                    0x0055 => {
                        let xindex = opcode.x() as usize + 1;

                        for i in 0..xindex {
                            self.memory[self.i as usize + i] = self.v[i];
                        }
                        // unsafe {
                        //     let src = self.v[0..xindex].as_ptr();
                        //     let dst = self.memory.as_mut_ptr().offset(self.i as isize);
                        //     std::ptr::copy_nonoverlapping(src, dst, xindex)
                        // }
                    },
                    // Fx65 - LD Vx, [I] - Read registers V0 through Vx from memory starting at location I.
                    0x0065 => {
                        let xindex = opcode.x() as usize + 1;

                        for i in 0..xindex {
                            self.v[i] = self.memory[self.i as usize + i];
                        }
                        // unsafe {
                        //     let src = self.memory.as_ptr().offset(self.i as isize);
                        //     let dst = self.v.as_mut_ptr();
                        //     std::ptr::copy_nonoverlapping(src, dst, xindex);
                        // }
                    }
                    _ => return Err(format_err(opcode))
                }               
            },
            _ => return Err(format_err(opcode))
        }

        if self.delay_timer > 0 { self.delay_timer -= 1; }
        if self.sound_timer > 0 { self.sound_timer -= 1; }
        Ok(should_draw)
    }
}
  

fn random_8bit() -> u8 {         
    let n: u32 = thread_rng().gen_range(0, 256);
    n as u8
}

fn format_err(Opcode(c): Opcode) -> String {
    format!("OPCODE: {} NOT VALID", c)
} 

