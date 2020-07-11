use rand::Rng;

const MEM_SIZE: usize = 4096;
const N_REGISTERS: usize = 16;
const STACK_SIZE: usize = 16;
const PROG_START: usize = 0x200;
const FONT_START: usize = 0;
const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const N_KEYS: usize = 16;
const OP_SIZE: usize = 2;

type Opcode = u16;

const FONTSET: [u8; 80] = [
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

pub struct Emulator {
    memory: [u8; MEM_SIZE],
    pub registers: [u8; N_REGISTERS],
    i: usize,
    pc: usize,
    graphics: Vec<Vec<u8>>,
    delay_timer: u8,
    sound_timer: u8,
    stack: [usize; STACK_SIZE],
    sp: usize,
    key_state: [bool; N_KEYS],
    await_key_target: Option<usize>,
    graphics_changed: bool,
}

impl Emulator {
    pub fn new() -> Self {
        let mut memory = [0; MEM_SIZE];
        for (i, &hex) in FONTSET.iter().enumerate() {
            memory[FONT_START + i] = hex;
        }

        Emulator {
            memory: memory,
            registers: [0; N_REGISTERS],
            i: 0,
            pc: PROG_START,
            graphics: (0..SCREEN_HEIGHT).map(|_| vec![0; SCREEN_WIDTH]).collect(),
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; STACK_SIZE],
            sp: 0,
            key_state: [false; N_KEYS],
            await_key_target: None,
            graphics_changed: false,
        }
    }

    pub fn load_program(&mut self, instructions: Vec<u8>) {
        for (i, &instruction) in instructions.iter().enumerate().take_while(|(i, _)| *i < PROG_START + MEM_SIZE) {
            self.memory[i + PROG_START] = instruction;
        }
    }

    pub fn graphics(&self) -> &Vec<Vec<u8>> {
        &self.graphics
    }

    pub fn graphics_changed(&self) -> bool {
        self.graphics_changed
    }

    pub fn run_cycle(&mut self) {
        self.graphics_changed = false;

        if let Some(reg_idx) = self.await_key_target {
            for (i, &key) in self.key_state.iter().enumerate() {
                if key {
                    self.registers[reg_idx] = i as u8;
                    self.await_key_target = None;
                    break;
                }
            }
        }
        else {
            let opcode = self.fetch_opcode(self.pc as usize);
            self.run_opcode(opcode);

            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }

            if self.sound_timer > 0 {
                if self.sound_timer == 1 {
                    // beep
                }
                self.sound_timer -= 1;
            }
        }
    }

    fn fetch_opcode(&self, address: usize) -> Opcode {
        (self.memory[address] as u16) << 8 | self.memory[address + 1] as u16
    }

    fn run_opcode(&mut self, opcode: Opcode) {
        let nnn = (opcode & (2u16.pow(12) - 1)) as usize;
        let nn = (opcode & (2u16.pow(8) - 1)) as u8;

        let mask = 0b1111;
        let opcode_nibbles = (
            ((opcode & (mask << 12)) >> 12) as usize,
            ((opcode & (mask << 8)) >> 8) as usize,
            ((opcode & (mask << 4)) >> 4) as usize,
            (opcode & mask) as usize
        );

        match opcode_nibbles {
            (0, 0, 0x0e, 0) => self.i_00e0(),
            (0, 0, 0x0e, 0x0e) => self.i_00ee(),
            (0x01, _, _, _) => self.i_1nnn(nnn),
            (0x02, _, _, _) => self.i_2nnn(nnn),
            (0x03, x, _, _) => self.i_3xnn(x, nn),
            (0x04, x, _, _) => self.i_4xnn(x, nn),
            (0x05, x, y, 0) => self.i_5xy0(x, y),
            (0x06, x, _, _) => self.i_6xnn(x, nn),
            (0x07, x, _, _) => self.i_7xnn(x, nn),
            (0x08, x, y, 0) => self.i_8xy0(x, y),
            (0x08, x, y, 0x01) => self.i_8xy1(x, y),
            (0x08, x, y, 0x02) => self.i_8xy2(x, y),
            (0x08, x, y, 0x03) => self.i_8xy3(x, y),
            (0x08, x, y, 0x04) => self.i_8xy4(x, y),
            (0x08, x, y, 0x05) => self.i_8xy5(x, y),
            (0x08, x, _, 0x06) => self.i_8xy6(x),
            (0x08, x, y, 0x07) => self.i_8xy7(x, y),
            (0x08, x, _, 0x0e) => self.i_8xye(x),
            (0x09, x, y, 0) => self.i_9xy0(x, y),
            (0x0a, _, _, _) => self.i_annn(nnn),
            (0x0b, _, _, _) => self.i_bnnn(nnn),
            (0x0c, x, _, _) => self.i_cxnn(x, nn),
            (0x0d, x, y, n) => self.i_dxyn(x, y, n),
            (0x0e, x, 0x09, 0x0e) => self.i_ex9e(x),
            (0x0e, x, 0x0a, 0x01) => self.i_exa1(x),
            (0x0f, x, 0, 0x07) => self.i_fx07(x),
            (0x0f, x, 0, 0x0a) => self.i_fx0a(x),
            (0x0f, x, 0x01, 0x05) => self.i_fx15(x),
            (0x0f, x, 0x01, 0x08) => self.i_fx18(x),
            (0x0f, x, 0x01, 0x0e) => self.i_fx1e(x),
            (0x0f, x, 0x02, 0x09) => self.i_fx29(x),
            (0x0f, x, 0x03, 0x03) => self.i_fx33(x),
            (0x0f, x, 0x05, 0x05) => self.i_fx55(x),
            (0x0f, x, 0x06, 0x05) => self.i_fx65(x),
            _ => panic!("Unknown Opcode: {}", opcode)
        }
    }

    fn i_00e0(&mut self) {
        for row in self.graphics.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = 0;
            }
        }
        self.graphics_changed = true;
        self.pc += OP_SIZE;
    }

    fn i_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp] + OP_SIZE;
    }

    fn i_1nnn(&mut self, nnn: usize) {
        self.pc = nnn;
    }

    fn i_2nnn(&mut self, nnn: usize) {
        self.stack[self.sp] = self.pc;
        self.sp += 1;
        self.pc = nnn;
    }

    fn i_3xnn(&mut self, x: usize, nn: u8) {
        if self.registers[x] == nn {
            self.pc += OP_SIZE;
        }
        self.pc += OP_SIZE;
    }

    fn i_4xnn(&mut self, x: usize, nn: u8) {
        if self.registers[x] != nn {
            self.pc += OP_SIZE;
        }
        self.pc += OP_SIZE;
    }

    fn i_5xy0(&mut self, x: usize, y: usize) {
        if self.registers[x] == self.registers[y] {
            self.pc += OP_SIZE;
        }
        self.pc += OP_SIZE;
    }

    fn i_6xnn(&mut self, x: usize, nn: u8) {
        self.registers[x] = nn;
        self.pc += OP_SIZE;
    }

    fn i_7xnn(&mut self, x: usize, nn: u8) {
        self.registers[x] =  truncated_add(self.registers[x], nn);
        self.pc += OP_SIZE;
    }

    fn i_8xy0(&mut self, x: usize, y: usize) {
        self.registers[x] = self.registers[y];
        self.pc += OP_SIZE;
    }

    fn i_8xy1(&mut self, x: usize, y: usize) {
        self.registers[x] |= self.registers[y];
        self.pc += OP_SIZE;
    }

    fn i_8xy2(&mut self, x: usize, y: usize) {
        self.registers[x] &= self.registers[y];
        self.pc += OP_SIZE;
    }

    fn i_8xy3(&mut self, x: usize, y: usize) {
        self.registers[x] ^= self.registers[y];
        self.pc += OP_SIZE;
    }

    fn i_8xy4(&mut self, x: usize, y: usize) {
        if self.registers[x].checked_add(self.registers[y]).is_none() {
            *self.vf() = 1;
        }
        else {
            *self.vf() = 0;
        }
        self.registers[x] = truncated_add(self.registers[x], self.registers[y]);
        self.pc += OP_SIZE;
    }

    fn i_8xy5(&mut self, x: usize, y: usize) {
        if self.registers[x] < self.registers[y] {
            *self.vf() = 0;
        }
        else {
            *self.vf() = 1;
        }
        self.registers[x] = self.registers[x].wrapping_sub(self.registers[y]);
        self.pc += OP_SIZE;
    }

    fn i_8xy6(&mut self, x: usize) {
        let lsb = 1 & self.registers[x];
        self.registers[x] >>= 1;
        *self.vf() = lsb;
        self.pc += OP_SIZE;
    }

    fn i_8xy7(&mut self, x: usize, y: usize) {
        if self.registers[x] < self.registers[y] {
            *self.vf() = 1;
        }
        else {
            *self.vf() = 0;
        }
        self.registers[x] = self.registers[y].wrapping_sub(self.registers[x]);
        self.pc += OP_SIZE;
    }

    fn i_8xye(&mut self, x: usize) {
        let msb = 1 & (self.registers[x] >> 7);
        self.registers[x] <<= 1;
        *self.vf() = msb;
        self.pc += OP_SIZE;
    }

    fn i_9xy0(&mut self, x: usize, y: usize) {
        if self.registers[x] != self.registers[y] {
            self.pc += OP_SIZE;
        }
        self.pc += OP_SIZE;
    }

    fn i_annn(&mut self, nnn: usize) {
        self.i = nnn;
        self.pc += OP_SIZE;
    }

    fn i_bnnn(&mut self, nnn: usize) {
        self.pc = nnn + self.registers[0] as usize
    }

    fn i_cxnn(&mut self, x: usize, nn: u8) {
        let value: u8 = rand::thread_rng().gen();
        self.registers[x] = nn & value;
        self.pc += OP_SIZE;
    }

    fn i_dxyn(&mut self, x: usize, y: usize, n: usize) {
        *self.vf() = 0;
        for y_offset in 0..n {
            let pixel = self.memory[self.i + y_offset];
            let p_y = (self.registers[y] as usize + y_offset) % SCREEN_HEIGHT;
            for x_offset in 0..8 {
                let bit = 1 & (pixel >> (7 - x_offset));
                let p_x = (self.registers[x] as usize + x_offset) % SCREEN_WIDTH;
                if bit == 1 && self.graphics[p_y][p_x] == bit{
                    *self.vf() = 1;
                }
                self.graphics[p_y][p_x] ^= bit;
            }
        }

        self.graphics_changed = true;
        self.pc += OP_SIZE;
    }

    fn i_ex9e(&mut self, x: usize) {
        if self.key_state[self.registers[x] as usize] {
            self.pc += OP_SIZE;
        }
        self.pc += OP_SIZE;
    }

    fn i_exa1(&mut self, x: usize) {
        if !self.key_state[self.registers[x] as usize] {
            self.pc += OP_SIZE;
        }
        self.pc += OP_SIZE;
    }

    fn i_fx07(&mut self, x: usize) {
        self.registers[x] = self.delay_timer;
        self.pc += OP_SIZE;
    }

    fn i_fx0a(&mut self, x: usize) {
        self.await_key_target = Some(x);
        self.pc += OP_SIZE;
    }

    fn i_fx15(&mut self, x: usize) {
        self.delay_timer = self.registers[x];
        self.pc += OP_SIZE;
    }

    fn i_fx18(&mut self, x: usize) {
        self.sound_timer = self.registers[x];
        self.pc += OP_SIZE;
    }

    fn i_fx1e(&mut self, x: usize) {
        self.i += self.registers[x] as usize;
        self.pc += OP_SIZE;
    }

    fn i_fx29(&mut self, x: usize) {
        self.i = FONT_START + (self.registers[x] * 5) as usize;
        self.pc += OP_SIZE;
    }

    fn i_fx33(&mut self, x: usize) {
        let vx = self.registers[x];
        self.memory[self.i] = vx / 100;
        self.memory[self.i + 1] = (vx % 100) / 10;
        self.memory[self.i + 2] = vx % 10;
        self.pc += OP_SIZE;
    }

    fn i_fx55(&mut self, x: usize) {
        for j in 0..=x {
            self.memory[self.i + j] = self.registers[j];
        }
        self.pc += OP_SIZE;
    }

    fn i_fx65(&mut self, x: usize) {
        for j in 0..=x {
            self.registers[j] = self.memory[self.i + j];
        }
        self.pc += OP_SIZE;
    }

    fn vf(&mut self) -> &mut u8 {
        &mut self.registers[0x0f]
    }
}

fn truncated_add(x: u8, y: u8) -> u8 {
    let z = (x as u16) + (y as u16);
    z as u8
}

#[cfg(test)]
mod tests {
    use super::{PROG_START, OP_SIZE, Emulator};

    #[test]
    fn test_5xy0() {
        let mut emulator = Emulator::new();
        emulator.registers[4] = 0x03;
        emulator.registers[5] = 0x04;
        emulator.registers[6] = 0x03;

        assert_eq!(emulator.pc, PROG_START);
        emulator.run_opcode(0x5450);
        assert_eq!(emulator.pc, PROG_START + OP_SIZE);
        emulator.run_opcode(0x5460);
        assert_eq!(emulator.pc, PROG_START + 3 * OP_SIZE);
    }

    #[test]
    fn test_dxyn_graphics_updates() {
        let mut emulator = Emulator::new();
        emulator.registers[2] = 5;
        emulator.registers[3] = 6;
        emulator.memory[0x500] = 0b10101100;
        // set i = 0x500
        emulator.run_opcode(0xa500);
        // update graphics, at screen position y=reg[2], x=reg[3]
        emulator.run_opcode(0xd231);

        let expected_row = vec![1, 0, 1, 0, 1, 1, 0, 0];
        assert_eq!(emulator.graphics[6][5..13], expected_row[..]);
        assert_eq!(*emulator.vf(), 0);

        emulator.memory[0x500] = 0b00110000;
        emulator.run_opcode(0xd231);
        let expected_row = vec![1, 0, 0, 1, 1, 1, 0, 0];
        assert_eq!(emulator.graphics[6][5..13], expected_row[..]);
        assert_eq!(*emulator.vf(), 1);
    }
}
