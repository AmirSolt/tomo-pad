
import Keyboard from 'simple-keyboard';
import 'simple-keyboard/build/css/index.css';
import { initNavigation, handleMove, handleSelect, getActiveKey } from './navigation';
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
 
const keyboard = new Keyboard({
  onChange: input => onChange(input),
  onKeyPress: button => onKeyPress(button),
  onInit: () => { refreshTabIndex(); setupButtonListeners(); },
  onRender: () => { refreshTabIndex(); setupButtonListeners(); },
  theme: "hg-theme-default myTheme1"
});

function refreshTabIndex() {
  document.querySelectorAll('.hg-button').forEach(btn => {
    btn.setAttribute('tabindex', '0');
  });
}

function setupButtonListeners() {
  const buttons = document.querySelectorAll('.hg-button');
  buttons.forEach(btn => {
    const buttonValue = btn.getAttribute('data-skbtn');
    if (!buttonValue) return;

    if (btn.hasAttribute('data-listener-attached')) return;
    btn.setAttribute('data-listener-attached', 'true');

    let repeatTimer: number | null = null;
    let isDown = false;

    const startRepeat = () => {
        if (repeatTimer) return;
        repeatTimer = window.setTimeout(() => {
            repeatTimer = window.setInterval(() => {
                sendKey(buttonValue, 'repeat');
            }, 75);
        }, 300);
    };

    const stopRepeat = () => {
        if (repeatTimer) {
            clearTimeout(repeatTimer);
            clearInterval(repeatTimer);
            repeatTimer = null;
        }
    };

    const onDown = (e: PointerEvent) => {
        if (isDown) return;
        isDown = true;
        sendKey(buttonValue, 'down');
        startRepeat();
        try {
            btn.setPointerCapture(e.pointerId);
        } catch (err) {
            // Ignore
        }
    };

    const onUp = (e: PointerEvent) => {
        if (!isDown) return;
        isDown = false;
        stopRepeat();
        sendKey(buttonValue, 'up');
        try {
            btn.releasePointerCapture(e.pointerId);
        } catch (err) {
            // Ignore
        }
    };
    
    btn.addEventListener('pointerdown', onDown as EventListener);
    btn.addEventListener('pointerup', onUp as EventListener);
    btn.addEventListener('pointercancel', onUp as EventListener);
  });
}

async function sendKey(key: string, phase: 'down' | 'repeat' | 'up') {
    const payload: any = { phase };
    if (key.startsWith('{') && key.endsWith('}')) {
        payload.key = key;
    } else {
        payload.text = key;
    }
    
    try {
        await invoke('send_key', { payload });
    } catch (e) {
        console.error("Failed to send key", e);
    }
}

initNavigation();
 
function onChange(input: string){
  const inputEl = document.querySelector(".input") as HTMLInputElement | null;
  if (inputEl) {
    inputEl.value = input;
  }
  console.log("Input changed", input);

}
 
function onKeyPress(button: string){
  console.log("Button pressed", button);

  if (button === "{shift}" || button === "{lock}") handleShift();
}

function handleShift() {
  const currentFocus = document.activeElement;
  const buttons = Array.from(document.querySelectorAll('.hg-button'));
  const focusIndex = buttons.indexOf(currentFocus as HTMLElement);

  let currentLayout = keyboard.options.layoutName;
  let shiftToggle = currentLayout === "default" ? "shift" : "default";

  keyboard.setOptions({
    layoutName: shiftToggle
  });

  if (focusIndex > -1) {
    const newButtons = document.querySelectorAll('.hg-button');
    const targetButton = newButtons[focusIndex] as HTMLElement;
    if (targetButton) targetButton.focus();
  }
}





listen('osk:nav:move', (event: any) => {
    const { phase, dx, dy } = event.payload;
    handleMove(phase, dx, dy);
});

let selectRepeatTimer: number | null = null;

listen('osk:nav:select', (event: any) => {
    const { phase } = event.payload;
    handleSelect(phase);
    
    const key = getActiveKey();
    if (!key) return;
    
    if (phase === 'down') {
        sendKey(key, 'down');
        if (selectRepeatTimer) return;
        selectRepeatTimer = window.setTimeout(() => {
            selectRepeatTimer = window.setInterval(() => {
                sendKey(key, 'repeat');
            }, 75);
        }, 300);
    } else if (phase === 'up') {
        sendKey(key, 'up');
        if (selectRepeatTimer) {
            clearTimeout(selectRepeatTimer);
            clearInterval(selectRepeatTimer);
            selectRepeatTimer = null;
        }
    }
});

listen('osk:nav:shift', () => {
    handleShift();
});

async function checkForAppUpdates() {
    try {
        const update = await check();
        if (update) {
            console.log(`[UPDATER] Found update ${update.version}`);
            await update.downloadAndInstall();
            console.log('[UPDATER] Update installed, relaunching...');
            await relaunch();
        } else {
            console.log('[UPDATER] No updates found');
        }
    } catch (error) {
        console.error('[UPDATER] Failed to check for updates:', error);
    }
}

checkForAppUpdates();
