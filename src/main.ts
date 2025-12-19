
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

function handleNavigation(e: KeyboardEvent) {
  if (e.key === 'Enter') {
      const active = document.activeElement as HTMLElement;
      if (active && active.classList.contains('key')) {
          active.click();
          active.classList.add('active');
          setTimeout(() => active.classList.remove('active'), 150);
      }
      return;
  }

  if (!['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(e.key)) return;

  const active = document.activeElement as HTMLElement;
  
  // If nothing is focused, or body is focused, focus the first key
  if (!active || active === document.body) {
      e.preventDefault();
      const firstKey = document.querySelector('[tabindex="0"]') as HTMLElement;
      if (firstKey) firstKey.focus();
      return;
  }
  
  if (active.getAttribute('tabindex') !== '0') return;

  e.preventDefault();

  const all = Array.from(document.querySelectorAll('[tabindex="0"]')) as HTMLElement[];
  const currentRect = active.getBoundingClientRect();
  const currentCenter = {
    x: currentRect.left + currentRect.width / 2,
    y: currentRect.top + currentRect.height / 2
  };

  let bestCandidate: HTMLElement | null = null;
  let minDistance = Infinity;

  let bestWrapCandidate: HTMLElement | null = null;
  let minWrapDistance = Infinity;

  for (const candidate of all) {
    if (candidate === active) continue;

    const rect = candidate.getBoundingClientRect();
    const center = {
      x: rect.left + rect.width / 2,
      y: rect.top + rect.height / 2
    };

    let isValid = false;
    if (e.key === 'ArrowRight') {
      isValid = center.x > currentCenter.x;
    } else if (e.key === 'ArrowLeft') {
      isValid = center.x < currentCenter.x;
    } else if (e.key === 'ArrowDown') {
      isValid = center.y > currentCenter.y;
    } else if (e.key === 'ArrowUp') {
      isValid = center.y < currentCenter.y;
    }

    if (isValid) {
        let primaryDist = 0;
        let secondaryDist = 0;
        
        if (e.key === 'ArrowLeft' || e.key === 'ArrowRight') {
            // Strict Y-axis overlap check for row navigation
            // The center of the current element MUST be within the vertical bounds of the candidate
            if (currentCenter.y < rect.top || currentCenter.y > rect.bottom) {
                isValid = false;
            } else {
                primaryDist = Math.abs(center.x - currentCenter.x);
                secondaryDist = 0;
            }
        } else {
            // Up / Down
            primaryDist = Math.abs(center.y - currentCenter.y);
            
            // Secondary: X axis overlap check
            if (currentCenter.x >= rect.left && currentCenter.x <= rect.right) {
                secondaryDist = 0;
            } else {
                secondaryDist = Math.min(
                    Math.abs(currentCenter.x - rect.left),
                    Math.abs(currentCenter.x - rect.right)
                );
            }
        }
        
        if (isValid) {
            const score = primaryDist + secondaryDist * 10;

            if (score < minDistance) {
                minDistance = score;
                bestCandidate = candidate;
            }
        }
    }

    // Wrapping Logic
    const isAlignedX = (rect.left < currentRect.right && rect.right > currentRect.left);
    const isAlignedY = (rect.top < currentRect.bottom && rect.bottom > currentRect.top);
    
    let isWrapCandidate = false;
    if (e.key === 'ArrowLeft' || e.key === 'ArrowRight') {
        isWrapCandidate = isAlignedY;
    } else {
        isWrapCandidate = isAlignedX;
    }

    if (isWrapCandidate) {
        let score = Infinity;
        // We want the element furthest in the OPPOSITE direction
        if (e.key === 'ArrowLeft') score = -center.x; // Maximize X -> Minimize -X
        else if (e.key === 'ArrowRight') score = center.x; // Minimize X
        else if (e.key === 'ArrowUp') score = -center.y; // Maximize Y -> Minimize -Y
        else if (e.key === 'ArrowDown') score = center.y; // Minimize Y
        
        if (score < minWrapDistance) {
            minWrapDistance = score;
            bestWrapCandidate = candidate;
        }
    }
  }

  if (bestCandidate) {
    bestCandidate.focus();
  } else if (bestWrapCandidate) {
    bestWrapCandidate.focus();
  }
}

document.addEventListener('click', (e) => {
    const target = e.target as HTMLElement;
    const keyElement = target.closest('.key') as HTMLElement;
    if (keyElement) {
        console.log("Pressed key:", keyElement.innerText || keyElement.id);
    }
});

window.addEventListener('keydown', handleNavigation);
