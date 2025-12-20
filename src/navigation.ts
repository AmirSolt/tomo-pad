
let activeElement: HTMLElement | null = null;
let moveInterval: number | null = null;

export function initNavigation() {
    // Initial selection
    setTimeout(() => {
        findInitialActive();
    }, 100); // Wait for render
}

function findInitialActive() {
    const first = document.querySelector('.hg-button') as HTMLElement;
    if (first) {
        setActive(first);
    }
}

export function handleMove(phase: 'down' | 'repeat' | 'up', dx: number, dy: number) {
    console.log(phase)
    if (phase === 'down') {
        move(dx, dy);
        startMoveRepeat(dx, dy);
    } else if (phase === 'up') {
        stopMoveRepeat();
    }
}

function startMoveRepeat(dx: number, dy: number) {
    stopMoveRepeat();
    moveInterval = window.setTimeout(() => {
        moveInterval = window.setInterval(() => {
            move(dx, dy);
        }, 100);
    }, 300);
}

function stopMoveRepeat() {
    if (moveInterval) {
        clearTimeout(moveInterval);
        clearInterval(moveInterval);
        moveInterval = null;
    }
}

function move(dx: number, dy: number) {
    if (!activeElement) {
        findInitialActive();
        if (!activeElement) return;
    }

    const all = Array.from(document.querySelectorAll('.hg-button')) as HTMLElement[];
    const currentRect = activeElement!.getBoundingClientRect();
    const currentCenter = {
        x: currentRect.left + currentRect.width / 2,
        y: currentRect.top + currentRect.height / 2
    };

    let bestCandidate: HTMLElement | null = null;
    let minDistance = Infinity;

    for (const candidate of all) {
        if (candidate === activeElement) continue;

        const rect = candidate.getBoundingClientRect();
        const center = {
            x: rect.left + rect.width / 2,
            y: rect.top + rect.height / 2
        };

        // Filter by direction
        let isValid = false;
        if (dx > 0) { // Right
            isValid = center.x > currentCenter.x;
        } else if (dx < 0) { // Left
            isValid = center.x < currentCenter.x;
        } else if (dy > 0) { // Down
            isValid = center.y > currentCenter.y;
        } else if (dy < 0) { // Up
            isValid = center.y < currentCenter.y;
        }

        if (!isValid) continue;

        // Calculate distance
        // Prefer candidates that are closer in the primary direction
        // and aligned in the other direction.
        
        const distSq = Math.pow(center.x - currentCenter.x, 2) + Math.pow(center.y - currentCenter.y, 2);
        
        // Weighting: penalize misalignment
        // If moving horizontal, penalize vertical distance
        let penalty = 0;
        if (dx !== 0) {
            penalty = Math.abs(center.y - currentCenter.y) * 5;
        } else {
            penalty = Math.abs(center.x - currentCenter.x) * 5;
        }
        
        const score = distSq + penalty * penalty;

        if (score < minDistance) {
            minDistance = score;
            bestCandidate = candidate;
        }
    }

    if (bestCandidate) {
        setActive(bestCandidate);
    }
}

function setActive(el: HTMLElement) {
    if (activeElement) {
        activeElement.classList.remove('active-key');
    }
    activeElement = el;
    activeElement.classList.add('active-key');
}

export function handleSelect(phase: 'down' | 'repeat' | 'up') {
    if (!activeElement) return;
    
    if (phase === 'down') {
        activeElement.classList.add('hg-activeButton');
    } else if (phase === 'up') {
        activeElement.classList.remove('hg-activeButton');
    }
}

export function getActiveKey(): string | null {
    return activeElement ? activeElement.getAttribute('data-skbtn') : null;
}
