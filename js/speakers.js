const BEEP_DURATION_MS = 200;
const WAVE_TYPE = 'square';

class Speakers {
  constructor() {
    this.ctx = new (window.AudioContext || window.webkitAudioContext || window.audioContext);
    this.volume = 1;
  }

  setVolume(enabled) {
    if (enabled) {
      this.volume = 1;
    }
    else {
      this.volume = 0;
    }
  }

  beep() {
    const oscillator = this.ctx.createOscillator();
    oscillator.type = WAVE_TYPE;
    const gainNode = this.ctx.createGain();
    gainNode.gain.value = this.volume;

    oscillator.connect(gainNode);
    gainNode.connect(this.ctx.destination);

    oscillator.start(this.ctx.currentTime);
    oscillator.stop(this.ctx.currentTime + BEEP_DURATION_MS / 1000);
  }
}

export default Speakers;
