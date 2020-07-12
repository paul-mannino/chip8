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

export default Screen;
