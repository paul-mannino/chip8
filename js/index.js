class Screen {
  constructor(canvasId) {
    this.canvas = document.getElementById(canvasId);
    this.ctx = this.canvas.getContext('2d');
    this.ctx.fillStyle = 'black';
  }

  render(pixels, logicalHeight, logicalWidth) {
    const sHeight = this.canvas.height;
    const sWidth = this.canvas.width;
    this.ctx.clearRect(0, 0, sWidth, sHeight);
    const pixelHeight = sHeight/logicalHeight;
    const pixelWidth = sWidth/logicalWidth;

    for (let j = 0; j < logicalHeight; j++) {
      for (let i = 0; i < logicalWidth; i++) {
        const xPos = i * pixelWidth;
        const yPos = j * pixelHeight;
        if (pixels[i + (j * logicalWidth)] == 1) {
          this.ctx.fillRect(xPos, yPos, pixelWidth, pixelHeight);
        }
      }
    }
  }
}

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

import("../pkg/index.js").then((pkg) => {
  const screen = new Screen('canvas');
  load_rom_bytes('IBM', function(instructions) {
    const bus = pkg.Bus.new();
    bus.load_rom(instructions)
    let i = 0;
    while(i < 100000) {
      i += 1;
      bus.clock_tick();
      if (bus.rerender()) {
        screen.render(
          bus.flattened_vram(),
          bus.vram_height(),
          bus.vram_width()
        )
      }
    }
  });
}).catch(console.error);

