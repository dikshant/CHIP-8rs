pub const CHIP8_WIDTH: usize = 64;
pub const CHIP8_HEIGHT: usize = 32;

// Display represents the CHIP-8 display
pub struct Display {
    screen: [[u8; CHIP8_HEIGHT]; CHIP8_WIDTH],

}

// Provides an implementation of CHIP-8 memory with the default sprites pre
// filled out in the appropriate pages
impl Display {
    pub fn new() -> Self {
        let mut display = Display {
            screen: [[0; CHIP8_HEIGHT]; CHIP8_WIDTH],
        };

        display
    }
}