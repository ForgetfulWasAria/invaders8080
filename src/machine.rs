pub struct Machine {
    memory: [u8; 0x4000],
}

impl Machine {
    pub fn new() -> Machine {
        Machine {
            memory: [0; 0x4000],
        }
    }
    pub fn read_byte(&mut self, address: u16) -> u8 {
        let location = (address as usize) % 0x4000;
        self.memory[location]
    }
    pub fn write_byte(&mut self, address: u16, value: u8) {
        let location = (address as usize) % 0x4000;
        self.memory[location] = value;
    }

    /// The execute method is called to start the 8080 emulation and does not return until emulation
    /// is finished.
    pub fn execute(&mut self) {}
}

impl Default for Machine {
    fn default() -> Self {
        Self::new()
    }
}
