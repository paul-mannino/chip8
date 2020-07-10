class Screen {
  constructor(canvasId) {
    this.canvas = document.getElementById(canvasId);
    this.ctx = this.canvas.getContext('2d');
    this.ctx.fillStyle = 'black';
  }

  render(pixels) {
    const lHeight = pixels.length;
    const lWidth = pixels[0].length;
    const sHeight = this.canvas.height;
    const sWidth = this.canvas.width;
    const pixelHeight = sHeight/lHeight;
    const pixelWidth = sWidth/lWidth;

    for (let j = 0; j < lHeight; j++) {
      for (let i = 0; i < lWidth; i++) {
        if (pixels[j][i] == 1) {
          this.ctx.fillRect(i * pixelWidth, j * pixelHeight, pixelWidth, pixelHeight);
        }
      }
    }
  }
}

import("../pkg/index.js").then((pkg) => {
  const screen = new Screen('canvas');
  screen.render([[1, 1, 1], [1, 0, 0], [0, 0, 1]]);
}).catch(console.error);

