extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

pub mod emulator;
use emulator::Emulator;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[wasm_bindgen]
pub struct Bus {
    emulator: Emulator
}

#[wasm_bindgen]
impl Bus {
    pub fn new() -> Self {
        Bus {
            emulator: Emulator::new()
        }
    }

    pub fn load_rom(&mut self, instructions: Vec<u8>) {
        self.emulator.load_program(instructions);
    }

    pub fn run_cycle(&mut self) {
        self.emulator.run_cycle();
    }

    pub fn rerender(&self) -> bool {
        self.emulator.graphics_changed()
    }

    pub fn flattened_vram(&self) -> Vec<u8> {
        self.emulator.graphics().iter().flatten().cloned().collect()
    }

    pub fn vram_height(&self) -> usize {
        self.emulator.graphics().len()
    }

    pub fn vram_width(&self) -> usize {
        self.emulator.graphics().len()
    }
}
