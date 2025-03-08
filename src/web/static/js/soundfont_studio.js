// DOM elements
const numHarmonicsSlider = document.getElementById('num-harmonics');
const numHarmonicsValue = document.getElementById('num-harmonics-value');
const harmonicsSliders = document.getElementById('harmonics-sliders');
const waveformCanvas = document.getElementById('waveform-canvas');
const previewBtn = document.getElementById('preview-btn');
const previewStatus = document.getElementById('preview-status');
const volumeSlider = document.getElementById('volume');
const volumeValue = document.getElementById('volume-value');
const previewFreqSlider = document.getElementById('preview-freq');
const previewFreqValue = document.getElementById('preview-freq-value');
const saveBtn = document.getElementById('save-btn');
const soundfontNameInput = document.getElementById('soundfont-name');
const saveSuccess = document.getElementById('save-success');
const saveError = document.getElementById('save-error');

// Preset buttons
const presetSineBtn = document.getElementById('preset-sine');
const presetSquareBtn = document.getElementById('preset-square');
const presetTriangleBtn = document.getElementById('preset-triangle');
const presetSawtoothBtn = document.getElementById('preset-sawtooth');
const presetOrganBtn = document.getElementById('preset-organ');
const presetClearBtn = document.getElementById('preset-clear');

// Audio context and nodes
let audioContext = null;
let oscillator = null;
let gainNode = null;
let isPlaying = false;
let currentHarmonics = [];
let currentVolume = 0.5; // 0-1 value
let previewFrequency = 440; // Default A4

// Initialize the UI
document.addEventListener('DOMContentLoaded', initializeUI);

/**
 * Initialize the UI components
 */
function initializeUI() {
    // Initialize audio context
    initAudio();

    // Set up the harmonics sliders
    updateNumHarmonics();

    // Set up event listeners
    numHarmonicsSlider.addEventListener('input', () => {
        updateNumHarmonicsValue();
        updateNumHarmonics();

        // Update the oscillator if playing
        if (isPlaying) {
            updateOscillator();
        }
    });

    volumeSlider.addEventListener('input', updateVolume);
    previewFreqSlider.addEventListener('input', updatePreviewFreq);
    previewBtn.addEventListener('click', togglePreview);

    // Preset buttons
    presetSineBtn.addEventListener('click', () => applyPreset('sine'));
    presetSquareBtn.addEventListener('click', () => applyPreset('square'));
    presetTriangleBtn.addEventListener('click', () => applyPreset('triangle'));
    presetSawtoothBtn.addEventListener('click', () => applyPreset('sawtooth'));
    presetOrganBtn.addEventListener('click', () => applyPreset('organ'));
    presetClearBtn.addEventListener('click', clearHarmonics);

    // Save button
    saveBtn.addEventListener('click', saveSoundfont);

    // Set up frequency input validations
    previewFreqValue.addEventListener('input', handlePreviewFreqInput);
    previewFreqValue.addEventListener('blur', commitPreviewFreqInput);

    // Update num harmonics value display
    updateNumHarmonicsValue();
    updatePreviewFreqValue();

    // Draw initial waveform
    drawWaveform();
}

/**
 * Initialize the audio context
 */
function initAudio() {
    try {
        audioContext = new (window.AudioContext || window.webkitAudioContext)();
    } catch (e) {
        console.error("Web Audio API is not supported in this browser", e);
    }
}

/**
 * Update the number of harmonics sliders
 */
function updateNumHarmonics() {
    const numHarmonics = parseInt(numHarmonicsSlider.value);

    // Clear existing sliders
    harmonicsSliders.innerHTML = '';

    // Resize the harmonics array if needed
    if (currentHarmonics.length < numHarmonics) {
        // Add zeros for new harmonics
        while (currentHarmonics.length < numHarmonics) {
            currentHarmonics.push(0);
        }
    } else if (currentHarmonics.length > numHarmonics) {
        // Truncate array to the new size
        currentHarmonics = currentHarmonics.slice(0, numHarmonics);
    }

    // Create sliders for each harmonic
    for (let i = 0; i < numHarmonics; i++) {
        const harmonicContainer = document.createElement('div');
        harmonicContainer.className = 'harmonic-slider';

        const label = document.createElement('label');
        label.textContent = `Harmonic ${i + 1}`;

        const slider = document.createElement('input');
        slider.type = 'range';
        slider.min = '0';
        slider.max = '1';
        slider.step = '0.01';
        slider.value = currentHarmonics[i] || 0;
        slider.id = `harmonic-${i + 1}`;
        slider.dataset.index = i;

        const valueDisplay = document.createElement('div');
        valueDisplay.className = 'value';
        valueDisplay.textContent = parseFloat(slider.value).toFixed(2);

        // Add event listener to update the harmonic value
        slider.addEventListener('input', (e) => {
            const index = parseInt(e.target.dataset.index);
            const value = parseFloat(e.target.value);
            currentHarmonics[index] = value;
            valueDisplay.textContent = value.toFixed(2);

            // Update the waveform visualization
            drawWaveform();

            // Update the oscillator if playing
            if (isPlaying) {
                updateOscillator();
            }
        });

        harmonicContainer.appendChild(label);
        harmonicContainer.appendChild(slider);
        harmonicContainer.appendChild(valueDisplay);
        harmonicsSliders.appendChild(harmonicContainer);
    }

    // Update the waveform visualization
    drawWaveform();
}

/**
 * Update the numeric display for the number of harmonics
 */
function updateNumHarmonicsValue() {
    numHarmonicsValue.value = numHarmonicsSlider.value;
}

/**
 * Update the volume display and oscillator volume
 */
function updateVolume() {
    currentVolume = volumeSlider.value / 100;
    volumeValue.textContent = `${volumeSlider.value}%`;

    if (gainNode) {
        gainNode.gain.value = currentVolume;
    }
}

/**
 * Update the preview frequency display and oscillator frequency
 */
function updatePreviewFreq() {
    previewFrequency = parseFloat(previewFreqSlider.value);
    previewFreqValue.value = previewFrequency.toFixed(0);

    if (oscillator) {
        // Update the oscillator frequency
        oscillator.frequency.setValueAtTime(previewFrequency, audioContext.currentTime);
    }
}

/**
 * Handle manual input for preview frequency
 */
function handlePreviewFreqInput(event) {
    const inputValue = event.target.value.trim();
    const numValue = parseFloat(inputValue);

    if (!isNaN(numValue) && numValue >= 50 && numValue <= 2000) {
        // Valid input
        event.target.classList.remove('invalid');
    } else {
        // Invalid input
        event.target.classList.add('invalid');
    }
}

/**
 * Commit the manually entered frequency value
 */
function commitPreviewFreqInput() {
    const inputValue = previewFreqValue.value.trim();
    const numValue = parseFloat(inputValue);

    if (!isNaN(numValue) && numValue >= 50 && numValue <= 2000) {
        // Valid input - update slider and oscillator
        previewFrequency = numValue;
        previewFreqSlider.value = numValue;

        if (oscillator) {
            oscillator.frequency.setValueAtTime(previewFrequency, audioContext.currentTime);
        }

        previewFreqValue.classList.remove('invalid');
    } else {
        // Reset to previous valid value
        previewFreqValue.value = previewFrequency.toFixed(0);
        previewFreqValue.classList.remove('invalid');
    }
}

/**
 * Update the preview frequency value display
 */
function updatePreviewFreqValue() {
    previewFreqValue.value = previewFreqSlider.value;
}

/**
 * Toggle sound preview on/off
 */
function togglePreview() {
    if (!audioContext) {
        initAudio();
    }

    if (isPlaying) {
        stopPreview();
    } else {
        startPreview();
    }
}

/**
 * Start the audio preview
 */
function startPreview() {
    if (isPlaying) return;

    if (audioContext.state === 'suspended') {
        audioContext.resume();
    }

    createOscillator();

    previewBtn.classList.add('playing');
    previewStatus.classList.remove('hidden');
    isPlaying = true;
}

/**
 * Stop the audio preview
 */
function stopPreview() {
    if (!isPlaying) return;

    if (oscillator) {
        oscillator.stop();
        oscillator.disconnect();
        oscillator = null;
    }

    if (gainNode) {
        gainNode.disconnect();
        gainNode = null;
    }

    previewBtn.classList.remove('playing');
    previewStatus.classList.add('hidden');
    isPlaying = false;
}

/**
 * Create and configure the oscillator
 */
function createOscillator() {
    if (!audioContext) return;

    // Create a gain node
    gainNode = audioContext.createGain();
    gainNode.gain.value = currentVolume;
    gainNode.connect(audioContext.destination);

    // Create a custom oscillator
    oscillator = audioContext.createOscillator();
    oscillator.frequency.value = previewFrequency;

    // Create a custom periodic wave
    updateOscillator();

    // Connect and start
    oscillator.connect(gainNode);
    oscillator.start();
}

/**
 * Update the oscillator with current harmonic values
 */
function updateOscillator() {
    if (!oscillator || !audioContext) return;

    try {
        // Create real and imag arrays for the periodicWave
        const real = new Float32Array(currentHarmonics.length + 1);
        const imag = new Float32Array(currentHarmonics.length + 1);

        // Set DC offset to 0
        real[0] = 0;
        imag[0] = 0;

        // Set harmonic values
        for (let i = 0; i < currentHarmonics.length; i++) {
            // Real part (cosine component)
            real[i + 1] = currentHarmonics[i];
            // Imaginary part (sine component) - set to 0 for phase alignment
            imag[i + 1] = 0;
        }

        // Create periodic wave
        const wave = audioContext.createPeriodicWave(real, imag, { disableNormalization: false });
        oscillator.setPeriodicWave(wave);
    } catch (e) {
        console.error("Error updating oscillator:", e);
    }
}

/**
 * Draw the waveform visualization
 */
function drawWaveform() {
    const canvas = waveformCanvas;
    const ctx = canvas.getContext('2d');
    const width = canvas.width;
    const height = canvas.height;

    // Clear canvas
    ctx.clearRect(0, 0, width, height);

    // Draw background and grid
    ctx.fillStyle = '#1e1e1e';
    ctx.fillRect(0, 0, width, height);

    // Draw center line
    ctx.strokeStyle = '#3a3a3a';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(0, height / 2);
    ctx.lineTo(width, height / 2);
    ctx.stroke();

    // Calculate waveform points
    const points = [];
    const numPoints = width;
    const waveformAmplitude = height * 0.2; // 20% of canvas height

    for (let i = 0; i < numPoints; i++) {
        const x = i;
        const t = (i / numPoints) * 2 * Math.PI;

        let y = 0;
        // Add up all harmonics
        for (let j = 0; j < currentHarmonics.length; j++) {
            const amplitude = currentHarmonics[j];
            const harmonic = j + 1;
            y += amplitude * Math.cos(harmonic * t);
        }

        // Scale and position in the middle of canvas
        y = (height / 2) - (y * waveformAmplitude);
        points.push({ x, y });
    }

    // Draw the waveform
    ctx.strokeStyle = '#00bfff';
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(points[0].x, points[0].y);

    for (let i = 1; i < points.length; i++) {
        ctx.lineTo(points[i].x, points[i].y);
    }

    ctx.stroke();
}

/**
 * Apply a preset waveform
 */
function applyPreset(presetName) {
    const numHarmonics = parseInt(numHarmonicsSlider.value);

    // Reset harmonics array
    currentHarmonics = Array(numHarmonics).fill(0);

    switch (presetName) {
        case 'sine':
            // Sine wave: just the fundamental
            currentHarmonics[0] = 1;
            break;

        case 'square':
            // Square wave: odd harmonics with 1/n amplitude
            for (let i = 0; i < numHarmonics; i++) {
                const harmonic = i + 1;
                if (harmonic % 2 !== 0) { // Odd harmonics only
                    currentHarmonics[i] = 1 / harmonic;
                }
            }
            break;

        case 'triangle':
            // Triangle wave: odd harmonics with alternating signs and 1/n² amplitude
            for (let i = 0; i < numHarmonics; i++) {
                const harmonic = i + 1;
                if (harmonic % 2 !== 0) { // Odd harmonics only
                    const sign = ((harmonic - 1) / 2 % 2 === 0) ? 1 : -1;
                    currentHarmonics[i] = sign / (harmonic * harmonic);
                }
            }
            break;

        case 'sawtooth':
            // Sawtooth wave: all harmonics with 1/n amplitude
            for (let i = 0; i < numHarmonics; i++) {
                const harmonic = i + 1;
                currentHarmonics[i] = 1 / harmonic;
            }
            break;

        case 'organ':
            // Organ-like sound: fundamental and some specific harmonics
            if (numHarmonics >= 1) currentHarmonics[0] = 1.0;    // Fundamental
            if (numHarmonics >= 2) currentHarmonics[1] = 0.6;    // 2nd harmonic
            if (numHarmonics >= 3) currentHarmonics[2] = 0.4;    // 3rd harmonic
            if (numHarmonics >= 4) currentHarmonics[3] = 0.2;    // 4th harmonic
            if (numHarmonics >= 5) currentHarmonics[4] = 0.3;    // 5th harmonic
            if (numHarmonics >= 8) currentHarmonics[7] = 0.1;    // 8th harmonic
            break;
    }

    // Update UI to reflect the new harmonic values
    updateHarmonicSliders();
    drawWaveform();

    // Update the oscillator if playing
    if (isPlaying) {
        updateOscillator();
    }
}

/**
 * Update all harmonic sliders to match currentHarmonics array
 */
function updateHarmonicSliders() {
    const sliders = harmonicsSliders.querySelectorAll('input[type="range"]');

    sliders.forEach((slider, index) => {
        const value = currentHarmonics[index];
        slider.value = value;
        slider.nextElementSibling.textContent = value.toFixed(2);
    });
}

/**
 * Clear all harmonics (set to zero)
 */
function clearHarmonics() {
    const numHarmonics = parseInt(numHarmonicsSlider.value);
    currentHarmonics = Array(numHarmonics).fill(0);

    updateHarmonicSliders();
    drawWaveform();

    if (isPlaying) {
        updateOscillator();
    }
}

/**
 * Save the current harmonics as a soundfont
 */
async function saveSoundfont() {
    hideMessages();

    const soundfontName = soundfontNameInput.value.trim();
    if (!soundfontName) {
        showError("Please enter a name for your soundfont");
        return;
    }

    try {
        // Round the values to 5 decimal places before saving
        const roundedHarmonics = currentHarmonics.map(value =>
            parseFloat(value.toFixed(5))
        );

        const response = await fetch(`/save-soundfont/${soundfontName}`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(roundedHarmonics)
        });

        if (!response.ok) {
            const errorText = await response.text();
            throw new Error(errorText || 'Failed to save soundfont');
        }

        showSuccess("Soundfont saved successfully!");
    } catch (error) {
        showError(error.message || "Error saving soundfont");
    }
}

/**
 * Show error message
 */
function showError(message) {
    saveError.textContent = message;
    saveError.classList.remove('hidden');

    // Auto-hide after 5 seconds
    setTimeout(() => {
        saveError.classList.add('hidden');
    }, 5000);
}

/**
 * Show success message
 */
function showSuccess(message) {
    saveSuccess.textContent = message;
    saveSuccess.classList.remove('hidden');

    // Auto-hide after 5 seconds
    setTimeout(() => {
        saveSuccess.classList.add('hidden');
    }, 5000);
}

/**
 * Hide all messages
 */
function hideMessages() {
    saveError.classList.add('hidden');
    saveSuccess.classList.add('hidden');
}