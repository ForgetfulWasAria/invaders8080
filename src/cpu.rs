#![allow(unused)]
use crate::machine;

pub struct CPU {
    pub(crate) a: u8,
    pub(crate) b: u8,
    pub(crate) c: u8,
    pub(crate) d: u8,
    pub(crate) e: u8,
    pub(crate) h: u8,
    pub(crate) l: u8,
    pub(crate) pc: u16,
    pub(crate) sp: u16,
    pub(crate) sign_flag: bool,
    pub(crate) zero_flag: bool,
    pub(crate) parity_flag: bool,
    pub(crate) carry_flag: bool,
    pub(crate) half_carry_flag: bool,
    pub(crate) interrupts_enabled: bool,
    pub(crate) bus: machine::Machine,
}

impl CPU {
    pub fn build(bus: machine::Machine) -> CPU {
        CPU {
            a: 0xFF,
            b: 0xFF,
            c: 0xFF,
            d: 0xFF,
            e: 0xFF,
            h: 0xFF,
            l: 0xFF,
            pc: 0,
            sp: 0xFFFF,
            sign_flag: true,
            zero_flag: true,
            parity_flag: true,
            carry_flag: true,
            half_carry_flag: true,
            interrupts_enabled: true,
            bus,
        }
    }

    // Register pairs.

    //
    pub(crate) fn get_bc(&self) -> u16 {
        ((self.b as u16) << 8) + (self.c as u16)
    }

    //
    pub(crate) fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = (value & 255) as u8;
    }

    //
    pub(crate) fn get_de(&self) -> u16 {
        ((self.d as u16) << 8) + (self.e as u16)
    }

    //
    pub(crate) fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = (value & 255) as u8;
    }

    //
    pub(crate) fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 255) as u8;
    }

    //
    pub(crate) fn get_hl(&self) -> u16 {
        ((self.h as u16) << 8) + (self.l as u16)
    }

    // Flag related methods.

    pub(crate) fn update_sign_flag(&mut self, result: u32) {
        self.sign_flag = (result as i32) < 0;
    }

    pub(crate) fn update_zero_flag(&mut self, result: u32) {
        self.zero_flag = result == 0;
    }

    pub(crate) fn update_parity_flag(&mut self, result: u32) {
        // Since parity is only calculated on 8 bit values, cast back to u8.
        let mut count = 0;
        for i in 0..=7 {
            if ((result as u8) & 2 ^ i) > 0 {
                count += 1
            }
        }
        self.parity_flag = count % 2 == 0;
    }

    pub(crate) fn update_carry_flag_16(&mut self, a: u16, b: u16) {
        self.carry_flag = (a as i32) + (b as i32) > (u16::MAX as i32);
    }

    pub(crate) fn update_carry_flag_8(&mut self, a: u8, b: u8) {
        self.carry_flag = (a as i32) + (b as i32) > 255;
    }

    pub(crate) fn update_half_carry(&mut self, a: u8, b: u8) {
        self.half_carry_flag = (a & 7) + (b & 7) > 7;
    }



    pub(crate) fn update_borrow_16(&mut self, a: u16, b: u16) {
        self.carry_flag = a < b;
    }

    pub(crate) fn update_borrow_8(&mut self, a: u8, b: u8) {
        self.carry_flag = a < b;
    }

    pub(crate) fn update_half_borrow(&mut self, a: u8, b: u8) {
        self.half_carry_flag = (a & 8) < (b & 8);
    }

    // Memory related methods

    pub(crate) fn read_8(&mut self, address: u16) -> u8 {
        self.bus.read_byte(address)
    }

    pub(crate) fn write_8(&mut self, address: u16, value: u8) {
        self.bus.write_byte(address, value);
    }

    pub(crate) fn fetch(&mut self) -> u8 {
        let value = self.bus.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        value
    }

    pub(crate) fn fetch_16(&mut self) -> u16 {
        let low = self.fetch();
        let high = self.fetch();
        ((high as u16) << 8) + low as u16
    }

    pub(crate) fn read_16(&mut self, address: u16) -> (u8, u8) {
        let low = self.bus.read_byte(address);
        let high = self.bus.read_byte(address + 1);
        (high, low)
    }

    pub(crate) fn write_16(&mut self, address: u16, value: (u8, u8)) {
        self.bus.write_byte(address, value.1);
        self.bus.write_byte(address + 1, value.0);
    }

    /// The base method for all 4 PUSH instructions. Exists mostly to prevent errors working with the stack.
    
    pub(crate) fn push(&mut self, value: (u8, u8)) {
        self.bus.write_byte(self.sp - 1, value.1);
        self.bus.write_byte(self.sp - 2, value.0);
        self.sp = self.sp.wrapping_sub(2);
    }
    /// The base method for all 4 POP instructions. Exists mostly to prevent errors working with the stack.
    pub(crate) fn pop(&mut self) -> (u8, u8) {
        let (high, low) = (self.bus.read_byte(self.sp + 1), self.bus.read_byte(self.sp));
        self.sp = self.sp.wrapping_add(2);
        (high, low)
    }

    /// Returns a byte containing the processor flags in the correct bit positions for the Program 
    /// Status word. 
    /// Flag                Bit
    /// sign_flag             7
    /// zero_flag             6
    /// Always 0              5
    /// half_carry_flag       4
    /// Always 0              3
    /// parity_flag           2
    /// Always 1              1
    /// carry_flag            0
    pub(crate) fn psw_get_flags(&mut self) -> u8 {
        ((self.sign_flag as u8) << 7) + 
        ((self.zero_flag as u8) << 6) +
            // Bit 5 is always clear.
        ((self.half_carry_flag as u8) << 4) +
            // Bit 3 is always clear.
        ((self.parity_flag as u8) << 2) +
            // Bit 1 is always set.
        2 +
        (self.carry_flag as u8)
    }

    /// Given the low byte of the PSW, sets the flags appropriately. Since bits 5, 3, and 1 are constant,
    /// they are discarded.
    pub(crate) fn psw_set_flags(&mut self, value: u8) {
        self.sign_flag = (value & 128) == 128;
        self.zero_flag = (value & 64) == 64;
        self.half_carry_flag = (value & 16) == 16;
        self.parity_flag = (value & 4) == 4;
        self.carry_flag = (value & 1) == 1;
    }
    
    pub(crate) fn handle_input(&mut self) {}

    pub(crate) fn handle_interrupts(&mut self) {}

}
