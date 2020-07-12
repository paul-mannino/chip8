const KEY_MAP = {
  '1': 0,
  '2': 1,
  '3': 2,
  '4': 3,
  'q': 4,
  'w': 5,
  'e': 6,
  'r': 7,
  'a': 8,
  's': 9,
  'd': 10,
  'f': 11,
  'z': 12,
  'x': 13,
  'c': 14,
  'v': 15
};

class Keyboard {
  constructor(bus) {
    this.bus = bus;
    this.keyState = new Array(16).fill(false);
  }

  loadListeners() {
    window.addEventListener("keyup", e => this.handleKeypress(e, false));
    window.addEventListener("keydown", e => this.handleKeypress(e, true));
    window.addEventListener("blur", () => this.clearKeys());
  }

  sendUserInput() {
    let pressedKeys = [];
    for (let i = 0; i < this.keyState.length; i++) {
      if (this.keyState[i]) {
        pressedKeys.push(i);
      }
    }

    if (pressedKeys.length > 0) {
      this.bus.send_pressed_keys(pressedKeys);
    }
  }

  handleKeypress(e, selected) {
    const key = e.key;
    if (key in KEY_MAP) {
      const keyIdx = KEY_MAP[key];
      this.keyState[keyIdx] = selected;
    }
  }

  clearKeys() {
    this.keyState.fill(false);
  }
}

export default Keyboard;
