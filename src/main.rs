use std::fs;
pub mod emulator;
use emulator::Emulator;
fn main() {
    let mut emu = Emulator::new();
    let filename = "./static/roms/TEST_CHIP8";
    let byz = std::fs::read(&filename).unwrap();
    emu.load_program(byz);
    for i in 0..2000 {
        emu.run_cycle();
        emu.run_cycle();
    }

    println!("Finished!")
}
