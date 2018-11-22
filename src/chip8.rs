use super::instruction::Instruction;
use rand;

pub const WIDTH: usize = 128;
pub const HEIGHT: usize = 64;
const MEMORY: usize = 4096;
const ROM_OFFSET: usize = 512;

const FONT_SET: [u8; 80] = [
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
    super_mode: bool,
    i: u16,
    pc: u16,
    mem: [u8; 4096],
    regs: [u8; 16],
    keypad: u16,
    display: [bool; WIDTH * HEIGHT],
    stack: [u16; 16],
    sp: u8,
    dt: u8
}

fn offset(x: usize, y: usize) -> usize {
    (x % WIDTH) + ((y % HEIGHT) * WIDTH)
}

impl Chip8 {
    pub fn new(data: &[u8]) -> Self {
        let mut mem = [0; MEMORY];
        mem[0..FONT_SET.len()].copy_from_slice(&FONT_SET);
        mem[ROM_OFFSET..(ROM_OFFSET + data.len())].copy_from_slice(data);

        Chip8 {
            super_mode: false,
            i: 0,
            pc: 0x0200,
            mem: mem,
            regs: [0; 16],
            keypad: 0,
            display: [false; WIDTH * HEIGHT],
            stack: [0; 16],
            sp: 0,
            dt: 0,
        }
    }

    pub fn get_display(&self) -> &[bool] {
        &self.display[..]
    }

    pub fn apply_keypad_value(&mut self, index: u8, pressed: bool) {
        assert!(index < 0x10, "index should be a nibble");

        if pressed {
            self.keypad |= 1 << index;
        } else {
            self.keypad &= !(1 << index);
        }
    }

    pub fn tick(&mut self) -> Instruction {
        let opcode = self.fetch();
        self.pc += 2;
        let instruction = Instruction::from_opcode(opcode);
        self.execute(&instruction);
        instruction
    }

    pub fn decrement_counter(&mut self) {
        if self.dt > 0 { self.dt -= 1 };
    }

    fn fetch(&self) -> u16 {
        (self.mem[self.pc as usize] as u16) << 8 | (self.mem[(self.pc + 1) as usize] as u16)
    }

    fn execute(&mut self, instruction: &Instruction) {
        use self::Instruction::*;

        match *instruction {
            ADDix { x } => self.i += self.regs[x] as u16,
            ADDxkk { x, kk } => self.regs[x] = self.regs[x].wrapping_add(kk),
            ADDxy { x, y } => { let (new, carry) = self.regs[x].overflowing_add(self.regs[y]); self.regs[x] = new; self.regs[0xF] = carry as u8 },
            AND { x, y } => self.regs[x] &= self.regs[y],
            CALL { nnn } => { self.stack[self.sp as usize] = self.pc; self.sp += 1; self.pc = nnn },
            CLS => self.display.copy_from_slice(&[false; WIDTH * HEIGHT]),
            DRW { x, y, n } => self.regs[0xF] = self.draw(x, y, n) as u8,
            DRWH { x, y } => self.regs[0xF] = self.draw_16(x, y) as u8,
            EXIT => panic!("EXIT not supported yet"), // ???
            INVALID { opcode } => panic!("opcode {:#X} not supported", opcode),
            HIGH => self.super_mode = true,
            JPnnn { nnn } => self.pc = nnn,
            JPnnnv { nnn } => self.pc = nnn + self.regs[0] as u16,
            LDbx { x } => self.mem[self.i as usize ..= self.i as usize + 2].copy_from_slice(&Self::get_bcd(self.regs[x])),
            LDfx { x } => self.i = self.regs[x] as u16 * 5,
            LDhfx { x } => self.i = self.regs[x] as u16 * 10,
            LDix { x } => for i in 0..=x { self.mem[self.i as usize + i] = self.regs[i] },
            LDnnn { nnn } => self.i = nnn,
            LDrx { .. } => panic!("LDrx not supported yet"), // ???
            LDsx { .. } => (), // Set sound timer register
            LDtx { x } => self.dt = self.regs[x],
            LDx { x } => if let Some(i) = self.check_keypad() { self.regs[x] = i } else { self.pc -= 2 },
            LDxi { x } => for i in 0..=x { self.regs[i] = self.mem[self.i as usize + i] },
            LDxkk { x, kk } => self.regs[x] = kk,
            LDxr { .. } => panic!("LDxr not supported yet"), // ???
            LDxt { x } => self.regs[x] = self.dt,
            LDxy { x, y } => self.regs[x] = self.regs[y],
            LOW => self.super_mode = false,
            OR { x, y } => self.regs[x] |= self.regs[y],
            RET => { self.sp -= 1; self.pc = self.stack[self.sp as usize] },
            RND { x, kk } => self.regs[x] = rand::random::<u8>() & kk,
            SCDn { n } => self.scroll_down(n),
            SCL => self.scroll_left(),
            SCR => self.scroll_right(),
            SExkk { x, kk } => if self.regs[x] == kk { self.pc += 2 },
            SExy { x, y } => if self.regs[x] == self.regs[y] { self.pc += 2 },
            SHL { x, .. } => { self.regs[0xF] = self.regs[x] >> 7; self.regs[x] <<= 1 },
            SHR { x, .. } => { self.regs[0xF] = self.regs[x] & 1; self.regs[x] >>= 1 },
            SKNP { x } => if self.keypad & (1 << self.regs[x]) == 0 { self.pc += 2 },
            SKP { x } => if self.keypad & (1 << self.regs[x]) != 0 { self.pc += 2 },
            SNExkk { x, kk } => if self.regs[x] != kk { self.pc += 2 },
            SNExy { x, y } => if self.regs[x] != self.regs[y] { self.pc += 2 },
            SUB { x, y } => { let (new, borrow) = self.regs[x].overflowing_sub(self.regs[y]); self.regs[x] = new; self.regs[0xF] = !borrow as u8 },
            SUBN { x, y } => { let (new, borrow) = self.regs[y].overflowing_sub(self.regs[x]); self.regs[x] = new; self.regs[0xF] = !borrow as u8 },
            XOR { x, y } => self.regs[x] ^= self.regs[y],
        }
    }

    fn draw(&mut self, x: usize, y: usize, n: u8) -> bool {
        let mut collision = false;

        for yoffset in 0..(n as usize) {
            for xoffset in 0..8 {
                if (self.mem[self.i as usize + yoffset] >> (7 - xoffset) & 0x01) == 1 {
                    let x = self.regs[x] as usize + xoffset;
                    let y = self.regs[y] as usize + yoffset;

                    if self.super_mode {
                        collision = collision | self.toggle_pixel(x, y);
                    } else {
                        collision = collision | self.toggle_pixel(x * 2, y * 2);
                        self.toggle_pixel(x * 2, y * 2 + 1);
                        self.toggle_pixel(x * 2 + 1, y * 2);
                        self.toggle_pixel(x * 2 + 1, y * 2 + 1);
                    }
                }
            }
        }

        collision
    }

    fn draw_16(&mut self, x: usize, y: usize) -> bool {
        if !self.super_mode {
            return false;
        }

        let mut collision = false;

        for yoffset in 0..16 {
            for xoffset in 0..16 {
                if (self.mem[self.i as usize + yoffset * 2 + (xoffset >> 3)] >> (7 - (xoffset & 0x07)) & 0x01) == 1 {
                    let x = self.regs[x] as usize + xoffset;
                    let y = self.regs[y] as usize + yoffset;

                    collision = collision | self.toggle_pixel(x, y);
                }
            }
        }

        collision
    }

    fn toggle_pixel(&mut self, x: usize, y: usize) -> bool {
        let pixel = &mut self.display[offset(x, y)];
        *pixel ^= true;
        !*pixel
    }

    fn scroll_down(&mut self, n: u8) {
        self.display.rotate_right(WIDTH * n as usize);
        for pixel in &mut self.display[0..(WIDTH * n as usize)] {
            *pixel = false;
        }
    }

    fn scroll_left(&mut self) {
        self.display.rotate_left(4);
        for y in 0..HEIGHT {
            for x in (WIDTH - 4)..WIDTH {
                self.display[offset(x, y)] = false;
            }
        }
    }

    fn scroll_right(&mut self) {
        self.display.rotate_right(4);
        for y in 0..HEIGHT {
            for x in 0..4 {
                self.display[offset(x, y)] = false;
            }
        }
    }

    fn check_keypad(&self) -> Option<u8> {
        let key = self.keypad.trailing_zeros() as u8;
        if key & 0xF == key {
            Some(key & 0xF)
        } else {
            None
        }
    }

    fn get_bcd(value: u8) -> [u8; 3] {
        [value / 100, value / 10 % 10, value % 10]
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::Instruction::*;

    #[test]
    fn apply_keypad_value() {
        let mut chip8 = Chip8::new(&[]);
        assert_eq!(chip8.keypad, 0b0000_0000);
        chip8.apply_keypad_value(0x0, true);
        assert_eq!(chip8.keypad, 0b0000_0001);
        chip8.apply_keypad_value(0x2, true);
        assert_eq!(chip8.keypad, 0b0000_0101);
        chip8.apply_keypad_value(0x0, false);
        assert_eq!(chip8.keypad, 0b0000_0100);
    }

    // #[test]
    // fn execute_shl() {
    //     let mut chip8 = Chip8::new(&[]);
    //     chip8.regs[1] = 0b0000_1111;
    //     chip8.regs[2] = 0b1000_0001;
    //     chip8.execute(&Instruction::SHL { x: 0, y: 1 });
    //     assert_eq!(chip8.regs[0], 0b0001_1110);
    //     assert_eq!(chip8.regs[0xf], 0);
    //     chip8.execute(&Instruction::SHL { x: 0, y: 2 });
    //     assert_eq!(chip8.regs[0], 0b0000_0010);
    //     assert_eq!(chip8.regs[0xf], 1);
    // }
    //
    // #[test]
    // fn execute_shr() {
    //     let mut chip8 = Chip8::new(&[]);
    //     chip8.regs[1] = 0b1010_1010;
    //     chip8.regs[2] = 0b1000_0001;
    //     chip8.execute(&Instruction::SHR { x: 0, y: 1 });
    //     assert_eq!(chip8.regs[0], 0b0101_0101);
    //     assert_eq!(chip8.regs[0xf], 0);
    //     chip8.execute(&Instruction::SHR { x: 0, y: 2 });
    //     assert_eq!(chip8.regs[0], 0b0100_0000);
    //     assert_eq!(chip8.regs[0xf], 1);
    // }

    #[test]
    fn execute_ldbx() {
        let mut chip8 = Chip8::new(&[]);
        chip8.regs[0] = 123;
        chip8.i = 456;
        chip8.execute(&LDbx { x: 0 });
        assert_eq!(chip8.mem[456], 1);
        assert_eq!(chip8.mem[457], 2);
        assert_eq!(chip8.mem[458], 3);
    }

    #[test]
    fn check_keypad() {
        let mut chip8 = Chip8::new(&[]);
        chip8.keypad = 0b0000_0000_0000_0000u16;
        assert_eq!(chip8.check_keypad(), None);
        chip8.keypad = 0b0000_0000_0000_0001u16;
        assert_eq!(chip8.check_keypad(), Some(0));
        chip8.keypad = 0b0000_0000_0000_1000u16;
        assert_eq!(chip8.check_keypad(), Some(3));
        chip8.keypad = 0b0000_0000_0000_1010u16;
        assert_eq!(chip8.check_keypad(), Some(1));
        chip8.keypad = 0b1010_1010_1010_1010u16;
        assert_eq!(chip8.check_keypad(), Some(1));
    }

    #[test]
    fn execute_drw() {
        let mut chip8 = Chip8::new(&[]);
        chip8.regs[0] = 2;
        chip8.regs[1] = 3;
        chip8.i = 456;
        chip8.mem[456] = 0b00111100;
        chip8.mem[457] = 0b01000010;
        chip8.mem[458] = 0b10000001;
        chip8.execute(&DRW { x: 0, y: 1, n: 3 });

        assert_eq!(chip8.display[3 * WIDTH + 2], false);
        assert_eq!(chip8.display[3 * WIDTH + 3], false);
        assert_eq!(chip8.display[3 * WIDTH + 4], true);
        assert_eq!(chip8.display[3 * WIDTH + 5], true);
        assert_eq!(chip8.display[3 * WIDTH + 6], true);
        assert_eq!(chip8.display[3 * WIDTH + 7], true);
        assert_eq!(chip8.display[3 * WIDTH + 8], false);
        assert_eq!(chip8.display[3 * WIDTH + 9], false);

        assert_eq!(chip8.display[4 * WIDTH + 2], false);
        assert_eq!(chip8.display[4 * WIDTH + 3], true);
        assert_eq!(chip8.display[4 * WIDTH + 4], false);
        assert_eq!(chip8.display[4 * WIDTH + 5], false);
        assert_eq!(chip8.display[4 * WIDTH + 6], false);
        assert_eq!(chip8.display[4 * WIDTH + 7], false);
        assert_eq!(chip8.display[4 * WIDTH + 8], true);
        assert_eq!(chip8.display[4 * WIDTH + 9], false);

        assert_eq!(chip8.display[5 * WIDTH + 2], true);
        assert_eq!(chip8.display[5 * WIDTH + 3], false);
        assert_eq!(chip8.display[5 * WIDTH + 4], false);
        assert_eq!(chip8.display[5 * WIDTH + 5], false);
        assert_eq!(chip8.display[5 * WIDTH + 6], false);
        assert_eq!(chip8.display[5 * WIDTH + 7], false);
        assert_eq!(chip8.display[5 * WIDTH + 8], false);
        assert_eq!(chip8.display[5 * WIDTH + 9], true);
    }

    #[test]
    fn execute_drw_collision() {
        let mut chip8 = Chip8::new(&[]);
        chip8.regs[0] = 2;
        chip8.regs[1] = 3;
        chip8.i = 456;
        chip8.mem[456] = 0b1000_0000;

        chip8.execute(&DRW { x: 0, y: 1, n: 1 });
        assert_eq!(chip8.regs[0xF], 0);

        chip8.execute(&DRW { x: 0, y: 1, n: 1 });
        assert_eq!(chip8.regs[0xF], 1);
    }
}
