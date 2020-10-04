use crate::memory;

const STACK_SIZE: usize = 16;

// CHIP8 is an interpreter to execute instructions
pub struct CHIP8 {
    // current opcode being executed
    opcode: u16,
    // the program counter
    pc: u16,
    // the stack pointer
    // http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.2
    sp: u8,
    // Memory index register
    ix: u8,
    // the input keypad
    keypad: [u8; 16],
    // the stack holds 16, 16 bit values
    stack: [u16; STACK_SIZE],
    // there are 16 v[0x0 to 0xf] registers, each capable of holding 8 bytes
    vx: [u8; 16],
    // memory page table is no larger than 4096 bytes with 8 byte pages
    // note: the first 512 bytes are reserved by the interpreter itself
    memory: memory::Memory,
    // display for this implementation is set to 32 x 64 (height x width) pixels
    display: [[u8; 32]; 64],
    // delay and sound timers that are decremented at the rate of 60Hz when > 0
    delay_timer: u8,
    sound_timer: u8,
}

// Yet another CHIP-8 emulator
impl CHIP8 {
    // new creates a new CHIP8 instance
    pub fn new() -> Self {
        let mut chip8 = CHIP8 {
            opcode: 0u16,
            pc: 0u16,
            sp: 0u8,
            ix: 0u8,
            keypad: [0u8; 16],
            stack: [0u16; 16],
            vx: [0; 16],
            memory: memory::Memory::new(),
            display: [[0; 32]; 64],
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
            if addr > 4096 {
                // TODO: error out if we try and load a program
                // that doesn't fit into memory?
                break;
            }
            self.memory.write(addr as u16, byte);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_load() {
        let mut chip = CHIP8::new();
        let data: [u8; 4] = [1, 2, 3, 4];
        chip.load(&data);
        for (i, &byte) in data.iter().enumerate() {
            assert_eq!(byte, chip.memory.read(0x200 + i as u16));
        }
    }
}
