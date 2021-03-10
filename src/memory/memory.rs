/// MAX_MEM_SIZE is fixed at 4KB
// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.1
pub const MAX_MEM_SIZE: usize = 4096;

// Memory represents the address spaces for the CHIP-8 interpreter
pub struct Memory {
    pages: [u8; MAX_MEM_SIZE],
}

// AccessError is a wrapper for different kinds of memory access errors
// that could be encountered
#[derive(Debug)]
pub enum AccessError {
    AddressOverflow,
    AddressUnderflow,
}

// sprites contains characters representing hex digits 0 through F
const SPRITES: [u8; 80] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

// Provides an implementation of CHIP-8 memory with the default sprites pre
// filled out in the appropriate pages
impl Memory {
    pub fn new() -> Self {
        let mut memory = Memory {
            pages: [0u8; MAX_MEM_SIZE],
        };
        // sprites are stored within 0x000 to 0x1FF
        for (i, val) in SPRITES.iter().enumerate() {
            memory.pages[i] = *val;
        }

        memory
    }

    // write will write a value at a particular address
    pub fn write(&mut self, address: u16, val: u8) -> Result<(), AccessError> {
        if address as usize > self.pages.len() - 1 {
            return Err(AccessError::AddressOverflow);
        }
        self.pages[address as usize] = val;
        Ok(())
    }

    // read will return the value at a particular address
    pub fn read(&mut self, address: u16) -> Result<u8, AccessError> {
        if address as usize > self.pages.len() - 1 {
            return Err(AccessError::AddressOverflow);
        }
        Ok(self.pages[address as usize])
    }
}
