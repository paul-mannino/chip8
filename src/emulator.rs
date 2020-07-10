use rand::Rng;

const MEM_SIZE: usize = 4096;
const N_REGISTERS: usize = 16;
const STACK_SIZE: usize = 16;
const PROG_START: usize = 0x200;
const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const N_KEYS: usize = 16;
const OP_SIZE: usize = 2;
const FONT_START: usize = 0x50;
const PROGRAM_START: usize = 0x200;

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
    registers: [u8; N_REGISTERS],
    i: usize,
    pc: usize,
    graphics: [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
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
            graphics: [[0; SCREEN_WIDTH]; SCREEN_HEIGHT],
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
        for (i, &instruction) in instructions.iter().enumerate() {
            self.memory[i + PROG_START] = instruction;
        }
    }

    pub fn run_cycle(&mut self) {
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
        let result = self.registers[x].wrapping_add(nn);
        self.registers[x] = result;
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
        let result = self.registers[x].wrapping_add(self.registers[y]);
        let carry;
        if result >= self.registers[x] && result >= self.registers[y] {
            carry = 0;
        }
        else {
            carry = 1;
        }
        self.registers[x] = result;
        self.registers[0x0f] = carry;
        self.pc += OP_SIZE;
    }

    fn i_8xy5(&mut self, x: usize, y: usize) {
        let borrow;
        if self.registers[x] < self.registers[y] {
            borrow = 1;
        }
        else {
            borrow = 0;
        }
        self.registers[x] = self.registers[x].wrapping_sub(self.registers[y]);
        self.registers[0x0f] = borrow;
        self.pc += OP_SIZE;
    }

    fn i_8xy6(&mut self, x: usize) {
        let lsb = 1 & self.registers[x];
        self.registers[x] >>= 1;
        self.registers[0x0f] = lsb;
        self.pc += OP_SIZE;
    }

    fn i_8xy7(&mut self, x: usize, y: usize) {
        let borrow;
        if self.registers[x] < self.registers[y] {
            borrow = 1;
        }
        else {
            borrow = 0;
        }
        self.registers[0x0f] = borrow;
        self.registers[x] = self.registers[y] - self.registers[x];
        self.pc += OP_SIZE;
    }

    fn i_8xye(&mut self, x: usize) {
        let msb = 1 & (self.registers[x] >> 7);
        self.registers[x] <<= 1;
        self.registers[0x0f] = msb;
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
        self.registers[0x0f] = 0;
        for y_offset in 0..n {
            let pixel = self.memory[self.i + y_offset];
            for x_offset in 0..8 {
                if (pixel & (1 << (7 - x_offset))) != 0 {
                    if self.graphics[y + y_offset][x + x_offset] == 1 {
                        self.registers[0x0f] = 1;
                    }
                    self.graphics[y + y_offset][x + x_offset] ^= 1;
                }
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
        self.i += x + 1;
        self.pc += OP_SIZE;
    }

    fn i_fx65(&mut self, x: usize) {
        for j in 0..=x {
            self.registers[j] = self.memory[self.i + j];
        }
        self.i += x + 1;
        self.pc += OP_SIZE;
    }
}
