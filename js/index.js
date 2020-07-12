import Screen from './screen';
import Keyboard from './keyboard';

const REFRESH_RATE_HZ = 200;

function load_rom_bytes(rom_name, callback) {
  const xhr = new XMLHttpRequest();
  xhr.open('GET', `roms/${rom_name}`);
  xhr.responseType = 'blob';
  xhr.onload = function () {
    let blob = xhr.response;
    const fileReader = new FileReader();
    fileReader.onload = function(event) {
      callback(new Uint8Array(event.target.result));
    }
    fileReader.readAsArrayBuffer(blob);
  }
  xhr.send();
}

function startRom(bus, screen, keyboard, instructions) {
  bus.load_rom(instructions)
  const cycle = function() {
    keyboard.sendUserInput();
    bus.clock_tick();
    if (bus.rerender_needed()) {
      screen.render(
        bus.flattened_vram(),
        bus.vram_height(),
        bus.vram_width()
      )
    }
  }

  setInterval(cycle, 1000/REFRESH_RATE_HZ);
}

import("../pkg/index.js").then((pkg) => {
  const screen = new Screen('canvas');
  const bus = pkg.Bus.new();
  const keyboard = new Keyboard(bus);
  keyboard.loadListeners();
  load_rom_bytes('PONG', instructions => startRom(bus, screen, keyboard, instructions));
}).catch(console.error);

