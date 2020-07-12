import Screen from './screen';
import Keyboard from './keyboard';

const REFRESH_RATE_HZ = 200;
const programRegistry = [];

function loadRomBytes(romName, callback) {
  const xhr = new XMLHttpRequest();
  xhr.open('GET', `roms/${romName}`);
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

function loadAndStart(program, bus, screen, keyboard) {
  loadRomBytes(program, instructions => startRom(bus, screen, keyboard, instructions));
}

const ROMS = [
  'PONG',
  'INVADERS',
  'MAZE',
  'UFO'
]

function setupRomSelector(bus, screen, keyboard) {
  const menu = document.getElementById('js-select-game');
  for (const rom of ROMS) {
    const option = document.createElement('option');
    option.value = rom;
    option.text = rom;
    menu.appendChild(option);
  }
  menu.addEventListener('change', function() {
    while (programRegistry.length > 0) {
      let runningProgram = programRegistry.pop();
      clearInterval(runningProgram);
    }
    const selectedRom = menu.value;
    loadAndStart(selectedRom, bus, screen, keyboard);
  })
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

  programRegistry.push(setInterval(cycle, 1000/REFRESH_RATE_HZ));
}

import("../pkg/index.js").then((pkg) => {
  const screen = new Screen('canvas');
  const bus = pkg.Bus.new();
  const keyboard = new Keyboard(bus);
  setupRomSelector(bus, screen, keyboard);
  keyboard.loadListeners();
  loadAndStart(ROMS[0], bus, screen, keyboard);
}).catch(console.error);

