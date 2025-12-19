
// const buttonStates: { [index: number]: boolean } = {};

// function pollGamepads() {
//   const gamepads = navigator.getGamepads();

//   for (const gamepad of gamepads) {
//     if (!gamepad) continue;

//     // Button 0 is usually 'A' on standard gamepads
//     const buttonA = gamepad.buttons[0];
//     const wasPressed = buttonStates[gamepad.index] || false;

//     if (buttonA && buttonA.pressed && !wasPressed) {
//       console.log("A button pressed");
//     }

//     if (buttonA) {
//       buttonStates[gamepad.index] = buttonA.pressed;
//     }
//   }
//   requestAnimationFrame(pollGamepads);
// }

// window.addEventListener("DOMContentLoaded", () => {
//   pollGamepads();
// });

// ===============
// The new method

import Keyboard from 'simple-keyboard';
import 'simple-keyboard/build/css/index.css';
import { handleNavigation } from './navigation';
 
const keyboard = new Keyboard({
  onChange: input => onChange(input),
  onKeyPress: button => onKeyPress(button),
  onInit: () => refreshTabIndex(),
  onRender: () => refreshTabIndex(),
  theme: "hg-theme-default myTheme1"
});

function refreshTabIndex() {
  document.querySelectorAll('.hg-button').forEach(btn => {
    btn.setAttribute('tabindex', '0');
  });
}

window.addEventListener('keydown', (e) => handleNavigation(e, keyboard));
 
function onChange(input: string){
  const inputEl = document.querySelector(".input") as HTMLInputElement | null;
  if (inputEl) {
    inputEl.value = input;
  }
  console.log("Input changed", input);

}
 
function onKeyPress(button: string){
  console.log("Button pressed", button);

  if (button === "{shift}" || button === "{lock}") handleShift(button);
}

function handleShift(button: string) {
  let currentLayout = keyboard.options.layoutName;
  let shiftToggle = currentLayout === "default" ? "shift" : "default";

  keyboard.setOptions({
    layoutName: shiftToggle
  });

  const btn = document.querySelector(`[data-skbtn="${button}"]`) as HTMLElement;
  if (btn) btn.focus();
}
