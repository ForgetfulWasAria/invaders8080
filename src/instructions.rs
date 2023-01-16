use crate::cpu::CPU;

impl CPU {
    pub(crate) fn do_instruction(&mut self) {
        let mut cycles = 0;
        let opcode = self.fetch();
        match opcode {
            // NOP
            0x00 => {
                cycles += 4;
            }
            // LXI B, D16
            0x01 => {
                let value = self.fetch_16();
                self.set_bc(value);
                cycles += 10;
            }
            // STAX B
            0x02 => {
                self.write_8(self.get_bc(), self.a);
                cycles += 7;
            }
            // INX B
            0x03 => {
                self.set_bc(self.get_bc().wrapping_add(1));
                cycles += 5;
            }
            // INR B
            0x04 => {
                self.b = self.inr(self.b);
                cycles += 5;
            }
            // DCR B
            0x05 => {
                self.b = self.dcr(self.b);
                cycles += 5;
            }
            // MVI B, D8
            0x06 => {
                self.b = self.fetch();
                cycles += 5;
            }
            // RLC
            0x07 => {
                self.carry_flag = self.a & 128 == 128;
                self.a <<= 1;
                if self.carry_flag {
                    self.a += 1
                };
                cycles += 4;
            }
            // Undocumented NOP
            0x08 => {
                cycles += 4;
            }
            // DAD B
            0x09 => {
                self.update_carry_flag_16(self.get_hl(), self.get_bc());
                let value = self.get_hl().wrapping_add(self.get_bc());
                self.set_hl(value);
                cycles += 10;
            }
            // LDAX B
            0x0a => {
                self.a = self.read_8(self.get_bc());
                cycles += 7;
            }
            // DCX B
            0x0b => {
                let value = self.get_bc().wrapping_sub(1);
                self.set_bc(value);
                cycles += 5;
            }
            // INR C
            0x0c => {
                self.c = self.inr(self.c);
                cycles += 5;
            }
            // DCR C
            0x0d => {
                self.c = self.dcr(self.c);
                cycles += 5;
            }
            // 	MVI C, D8
            0x0e => {
                self.c = self.fetch();
                cycles += 7;
            }
            // RRC
            0x0f => {
                self.carry_flag = self.a & 1 == 1;
                self.a >>= 1;
                if self.carry_flag {
                    self.a += 128
                };
                cycles += 4;
            }
            // Undocumented NOP
            0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 => {
                cycles += 4;
            }
            // LXI D, D16
            0x11 => {
                let value = self.fetch_16();
                self.set_de(value);
                cycles += 10;
            }
            // STAX D
            0x12 => {
                self.write_8(self.get_de(), self.a);
                cycles += 7;
            }
            // INX D
            0x13 => {
                self.set_de(self.get_de().wrapping_add(1));
                cycles += 5;
            }
            // INR D
            0x14 => {
                self.d = self.inr(self.d);
                cycles += 5;
            }
            // DCR D
            0x15 => {
                self.d = self.dcr(self.d);
                cycles += 5;
            }
            // MVI D, D8
            0x16 => {
                self.d = self.fetch();
                cycles += 7;
            }
            // RAL
            0x17 => {
                self.carry_flag = self.a & 128 == 128;
                self.a <<= 1;
                if self.carry_flag {
                    self.a += 1
                };
                cycles += 4;
            }
            _ => panic!("Unimplemented instruction at {} opcode {}", self.pc, opcode),
        }
    }

    pub(crate) fn inr(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1) as u32;
        self.update_sign_flag(result);
        self.update_zero_flag(result);
        self.update_parity_flag(result);
        self.update_half_carry(value, 1);
        result as u8
    }

    pub(crate) fn dcr(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1) as u32;
        self.update_sign_flag(result);
        self.update_zero_flag(result);
        self.update_parity_flag(result);
        self.update_half_borrow(value, 1);
        result as u8
    }

    pub(crate) fn decode(&mut self, opcode: u8) -> (u8, u8, u8, u8, u8) {
        let x = (opcode & 192) >> 5;
        let y = (opcode & 56) >> 3;
        let z = (opcode & 7);
        let p = (y & 6) >> 1;
        let q = y & 1;
        (x, y, z, p, q)
    }

    /// This is to handle the MOV instructions from 0x40 to 0x7F.
    pub(crate) fn mov(&mut self, opcode: u8) {
        let (_, y, z, _, _) = self.decode(opcode);
        let source = match z {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            7 => self.a,
            _ => 0,
        };
        match (z, y) {
            // Covers 0x76 HLT
            (6, 6) => self.pc = self.pc.wrapping_sub(2),
            // Covers all of the MOV reg, M which get the memory byte pointed at by the value in HL.
            (6, _) => self.bus.write_byte(self.get_hl(), source),
            (0, 6) => self.b = self.bus.read_byte(self.get_hl()),
            (1, 6) => self.c = self.bus.read_byte(self.get_hl()),
            (2, 6) => self.d = self.bus.read_byte(self.get_hl()),
            (3, 6) => self.e = self.bus.read_byte(self.get_hl()),
            (4, 6) => self.h = self.bus.read_byte(self.get_hl()),
            (5, 6) => self.l = self.bus.read_byte(self.get_hl()),
            (7, 6) => self.a = self.bus.read_byte(self.get_hl()),
            // This covers the remaining 49 MOV instructions.
            (0, _) => self.b = source,
            (1, _) => self.c = source,
            (2, _) => self.d = source,
            (3, _) => self.e = source,
            (4, _) => self.h = source,
            (5, _) => self.l = source,
            (7, _) => self.a = source,
            // This should never be executed.
            _ => {}
        }
    }
}
