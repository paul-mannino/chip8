extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;
extern crate console_error_panic_hook;
use std::panic;

use crate::emulator::Emulator;

#[wasm_bindgen]
pub struct Bus {
    emulator: Emulator
}

#[wasm_bindgen]
impl Bus {
    pub fn new() -> Self {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        Bus {
            emulator: Emulator::new()
        }
    }

    pub fn load_rom(&mut self, instructions: Vec<u8>) {
        self.emulator.load_program(instructions);
    }

    pub fn send_pressed_keys(&mut self, pressed_keys: Vec<usize>) {
        for key in pressed_keys.into_iter() {
            self.emulator.press_key(key);
        }
    }

    pub fn clock_tick(&mut self) {
        self.emulator.run_cycle();
    }

    pub fn rerender_needed(&self) -> bool {
        self.emulator.graphics_changed
    }

    pub fn flattened_vram(&self) -> Vec<u8> {
        self.emulator.graphics().iter().flatten().cloned().collect()
    }

    pub fn vram_height(&self) -> usize {
        self.emulator.graphics().len()
    }

    pub fn vram_width(&self) -> usize {
        if let Some(row) = self.emulator.graphics().first() {
            row.len()
        }
        else {
            0
        }
    }
}
