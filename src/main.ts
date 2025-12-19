

const buttonStates: { [index: number]: boolean } = {};

function pollGamepads() {
  const gamepads = navigator.getGamepads();

  for (const gamepad of gamepads) {
    if (!gamepad) continue;

    // Button 0 is usually 'A' on standard gamepads
    const buttonA = gamepad.buttons[0];
    const wasPressed = buttonStates[gamepad.index] || false;

    if (buttonA && buttonA.pressed && !wasPressed) {
      console.log("A button pressed");
    }

    if (buttonA) {
      buttonStates[gamepad.index] = buttonA.pressed;
    }
  }
  requestAnimationFrame(pollGamepads);
}

window.addEventListener("DOMContentLoaded", () => {
  pollGamepads();
});
