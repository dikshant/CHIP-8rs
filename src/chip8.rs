use crate::memory;
use crate::display;
use rand::Rng;

const STACK_SIZE: usize = 16;

// CHIP8 is an interpreter to execute instructions
pub struct CHIP8 {
    // the program counter
    pc: u16,
    // the stack pointer
    // http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.2
    sp: u8,
    // memory index register
    ix: u8,
    // the input keypad
    keypad: [u8; 16],
    // the stack holds 16, 16 bit values
    stack: [u16; STACK_SIZE],
    // there are 16 v[0x0 to 0xf] registers, each capable of holding 8 bytes
    vx: [u8; 16],
    // memory page table is no larger than 4096 bytes with 1 byte pages
    // note: the first 512 bytes are reserved by the interpreter itself
    memory: memory::Memory,
    // display for this implementation is set to 32 x 64 (height x width) pixels
    display: display::Display,
    // delay and sound timers that are decremented at the rate of 60Hz when > 0
    delay_timer: u8,
    sound_timer: u8,
}

// Yet another CHIP-8 emulator
impl CHIP8 {
    // new creates a new CHIP8 instance
    pub fn new() -> Self {
        let chip8 = CHIP8 {
            pc: 0u16,
            sp: 0u8,
            ix: 0u8,
            keypad: [0u8; 16],
            stack: [0u16; 16],
            vx: [0; 16],
            memory: memory::Memory::new(),
            display: display::Display::new(),
            delay_timer: 0u8,
            sound_timer: 0u8,
        };

        chip8
    }

    // load will load the contents of a CHIP8 program into memory
    pub fn load(&mut self, data: &[u8]) {
        for (i, &byte) in data.iter().enumerate() {
            // Programs start at 0x200 in memory
            let addr = 0x200 + i;
            if addr > memory::MAX_MEM_SIZE - 1 || addr < 0x200 {
                // TODO: error out if we try and load a program
                // that doesn't fit into memory?
                break;
            }
            match self.memory.write(addr as u16, byte) {
                Ok(_) => (),
                Err(_) => panic!("failed to write contents to memory"),
            };
        }
    }

    // get's the opcode using the program counter
    fn fetch_opcode(&mut self) -> u16 {
        // First we fetch the opcode from memory
        // Since each page is 1 byte and an opcode is two bytes
        // we need to grab the two consecutive memory addresses and
        // combine them to create the opcode
        let oc_first = match self.memory.read(self.pc) {
            Ok(oc_first) => oc_first,
            Err(_) => panic!("failed to read contents from memory"),
        } as u16;
        let oc_second = match self.memory.read(self.pc + 1) {
            Ok(oc_second) => oc_second,
            Err(_) => panic!("failed to read contents from memory"),
        } as u16;
        let opcode: u16 = (oc_first << 8 | oc_second).into();
        opcode
    }

    fn execute_opcode(&mut self, opcode: u16) {
        let ins = opcode & 0xF000;
        let nnn = (opcode & 0x0FFF) as u8;
        let nn = (opcode & 0x0FF0) as u8;
        let kk = (opcode & 0x00FF) as u8;
        let x = (opcode & 0x0F00) as u8;
        let y = (opcode & 0x00F0) as u8;
        let n = (opcode & 0x000F) as u8;
        // first we determine the top level instruction
        match ins {
            0x000 => {
                // disambiguate the opcode by comparing the last 4 bits
                match opcode & 0x000F {
                    0x0000 => self.op_00e0(), // Execute 00E0,
                    0x000E => self.op_00ee(), // Execute 00EE,
                    _ => (),
                }
            }
            0x100 => self.op_1nnn(nnn),
            0x200 => self.op_2nnn(nnn),
            0x300 => self.op_3xkk( x, kk),
            0x400 => self.op_4xkk( x, kk),
            0x500 => self.op_5xy0(x, y),
            0x600 => self.op_6xkk(x, kk),
            0x700 => self.op_7xkk(x, kk),
            0x800 => {
                match opcode & 0x000F {
                    0x800 => self.op_8xy0(x, y),
                    0x801 => self.op_8xy1(x, y),
                    0x802 => self.op_8xy2(x, y),
                    0x803 => self.op_8xy3(x, y),
                    0x804 => self.op_8xy4(x, y),
                    0x805 => self.op_8xy5(x, y),
                    0x806 => self.op_8xy6(x),
                    0x807 => self.op_8xy7(x, y),
                    0x80E => self.op_8xye(x),
                    _=> (),
                }
            }
            0x900 => self.op_9xy0(x, y),
            0xa00 => self.op_annn(nnn),
            0xb00 => self.op_bnnn(nnn),
            0xc00 => self.op_cxkk(x, kk),
            _ => (),
        }
    }

    // clear display
    fn op_00e0(&mut self) {}

    // return from a subroutine
    fn op_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize]
    }

    // jump to address
    fn op_1nnn(&mut self, nnn: u8) {
        self.pc = nnn as u16
    }

    // call address
    fn op_2nnn(&mut self, nnn: u8) {
        // first we increment the stack pointer
        self.sp = self.sp + 1;
        // then we put the current program counter on top of the stack
        // also make sure that we add the size of an opcode (2 bytes) to
        // the program counter
        self.stack[self.sp as usize] = self.pc + 2;
        // then we finally set the program counter to nnn
        self.pc = nnn as u16;
    }

    // skip instruction if Vx == kk and instruction is 3xkk
    fn op_3xkk(&mut self, x: u8, kk: u8) {
        if self.vx[x as usize] == kk {
            self.pc = self.pc + 4;
        }
    }

    // skip instruction if Vx != kk and instruction is 4xkk
    fn op_4xkk(&mut self, x: u8, kk: u8) {
        if self.vx[x as usize] != kk {
            self.pc = self.pc + 4;
        }
    }

    // skip next instruction if Vx == Vy
    fn op_5xy0(&mut self, x: u8, y: u8) {
        if self.vx[x as usize] == self.vx[y as usize] {
            self.pc = self.pc + 4;
        }
    }

    // puts the value of kk into register Vx
    fn op_6xkk(&mut self, x: u8, kk: u8) {
        self.vx[x as usize] = kk;
        self.pc = self.pc + 2;
    }

    // adds the value of kk into Vx and stores the result in Vx
    fn op_7xkk(&mut self, x: u8, kk: u8) {
        self.vx[x as usize] = self.vx[x as usize] | kk;
        self.pc = self.pc + 2;
    }

    // stores the value of register Vy in register Vx.
    fn op_8xy0(&mut self, x: u8, y: u8) {
        self.vx[x as usize] = self.vx[y as usize];
        self.pc = self.pc + 2;
    }

    // perform a bitwise OR on the values of Vx and Vy, then stores the result in Vx
    fn op_8xy1(&mut self, x: u8, y: u8) {
        self.vx[x as usize] |= self.vx[y as usize];
        self.pc = self.pc + 2;
    }

    // perform a bitwise OR on the values of Vx and Vy, then stores the result in Vx
    fn op_8xy2(&mut self, x: u8, y: u8) {
        self.vx[x as usize] &= self.vx[y as usize];
        self.pc = self.pc + 2;
    }

    // perform a bitwise OR on the values of Vx and Vy, then stores the result in Vx
    fn op_8xy3(&mut self, x: u8, y: u8) {
        self.vx[x as usize] ^= self.vx[y as usize];
        self.pc = self.pc + 2;
    }

    // sum the values of Vx and Vy, then store the result in Vx, if the sum > 8 bits
    // set VF to 1 otherwise 0
    fn op_8xy4(&mut self, x: u8, y: u8) {
        let sum :u16 = self.vx[x as usize] as u16 + self.vx[y as usize] as u16;
        self.vx[0x0f] = if sum > 0xFF { 1 } else { 0 };
        self.vx[x as usize] = sum as u8;
        self.pc = self.pc + 2;
    }

    // sub the value of Vy from Vx, then store the result in Vx, if the Vx > Vy
    // set VF to 1 otherwise 0
    fn op_8xy5(&mut self, x: u8, y: u8) {
        self.vx[0x0f] = if self.vx[x as usize] > self.vx[y as usize] { 1 } else {0};
        self.vx[x as usize] = self.vx[x as usize].wrapping_sub(self.vx[y as usize]);
        self.pc = self.pc + 2;
    }

    // set VF to 1 if LSB of Vx is 1, otherwise 0, then divide Vx by 2
    fn op_8xy6(&mut self, x: u8) {
        self.vx[0x0f] = self.vx[x as usize] & 0b10000000;
        self.vx[x as usize] >>= 1;
        self.pc = self.pc + 2;
    }

    // sub the value of Vx from Vy, then store the result in Vx, if the Vy > Vx
    // set VF to 1 otherwise 0
    fn op_8xy7(&mut self, x: u8, y: u8) {
        self.vx[0x0f] = if self.vx[y as usize] > self.vx[x as usize] { 1 } else { 0 };
        self.vx[x as usize] = self.vx[y as usize].wrapping_sub(self.vx[x as usize]);
        self.pc = self.pc + 2;
    }

     // set VF to 1 if LSB of Vx is 1, otherwise 0, then multiply Vx by 2
    fn op_8xye(&mut self, x: u8) {
        self.vx[0x0f] = self.vx[x as usize] & 0b10000000;
        self.vx[x as usize] >>= 1;
        self.pc = self.pc + 2;
    }

    // skip next instruction if Vx != Vy.
    fn op_9xy0(&mut self, x: u8, y:u8) {
        self.pc = if self.vx[x as usize] != self.vx[y as usize] {self.pc + 4} else {return};
    }

    // sets the value of the I register to nnn.
    fn op_annn(&mut self, nnn: u8) {
        self.ix = nnn;
    }

    // jumps to the location of nnn + v0
    fn op_bnnn(&mut self, nnn: u8) {
        self.pc = (nnn + self.vx[0]).into();
    }

    // sets vx to random byte AND kk
    fn op_cxkk(&mut self, x: u8, kk : u8) {
        let mut rng = rand::thread_rng();
        self.vx[x as usize] = rng.gen::<u8>() & kk;
    }
}

#[cfg(test)]
mod tests {
    use crate::chip8::*;
    #[test]
    fn test_load() {
        let mut chip = CHIP8::new();
        let data: [u8; 4] = [1, 2, 3, 4];
        chip.load(&data);
        for (i, &want) in data.iter().enumerate() {
            let got = match chip.memory.read(0x200 + i as u16) {
                Ok(read) => read,
                Err(_) => panic!("failed to read contents from memory"),
            };
            assert_eq!(want, got);
        }
    }
}
