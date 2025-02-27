import * as fileManager from './modules/fileManager.js';

// DOM elements
const uploadArea = document.getElementById('upload-area');
const fileInput = document.getElementById('file-input');
const uploadError = document.getElementById('upload-error');
const uploadSuccess = document.getElementById('upload-success');
const analyzeBtn = document.getElementById('analyze-btn');
const resultArea = document.getElementById('result-area');
const saveBtn = document.getElementById('save-btn');
const previewBtn = document.getElementById('preview-btn');
const previewStatus = document.getElementById('preview-status');
const volumeControl = document.getElementById('volume');
const volumeValue = document.getElementById('volume-value');
const volumeContainer = document.querySelector('.volume-control');

// Parameter elements
const samplesSlider = document.getElementById('samples');
const samplesValue = document.getElementById('samples-value');
const startTimeSlider = document.getElementById('start-time');
const startTimeValue = document.getElementById('start-time-value');
const baseFreqSlider = document.getElementById('base-freq');
const baseFreqValue = document.getElementById('base-freq-value');
const harmonicsSlider = document.getElementById('harmonics');
const harmonicsValue = document.getElementById('harmonics-value');

// Audio context and nodes
let audioContext = null;
let oscillator = null;
let gainNode = null;
const PREVIEW_FREQUENCY = 440; // Fixed A4 note for preview

// State
let uploadedFilename = null;
let currentHarmonics = null;
let isPlaying = false;
let debounceTimeout = null;
let currentVolume = 0.5; // Store volume as 0-1 value

// Event listeners
uploadArea.addEventListener('click', () => fileInput.click());
uploadArea.addEventListener('dragover', handleDragOver);
uploadArea.addEventListener('dragleave', handleDragLeave);
uploadArea.addEventListener('drop', handleDrop);
fileInput.addEventListener('change', handleFileSelect);
analyzeBtn.addEventListener('click', () => analyzeWav(false));
saveBtn.addEventListener('click', saveSoundfont);
previewBtn.addEventListener('click', togglePreview);

// Parameter update listeners with debounce
samplesSlider.addEventListener('input', () => {
    updateSamplesValue();
    debounceAnalysis();
});
startTimeSlider.addEventListener('input', () => {
    updateStartTimeValue();
    debounceAnalysis();
});
baseFreqSlider.addEventListener('input', () => {
    updateBaseFreqValue();
    debounceAnalysis();
});
harmonicsSlider.addEventListener('input', () => {
    updateHarmonicsValue();
    debounceAnalysis();
});

// Volume control
volumeControl.addEventListener('input', updateVolume);

// Functions
function handleDragOver(e) {
    e.preventDefault();
    uploadArea.classList.add('highlight');
}

function handleDragLeave(e) {
    e.preventDefault();
    uploadArea.classList.remove('highlight');
}

function handleDrop(e) {
    e.preventDefault();
    uploadArea.classList.remove('highlight');

    const files = e.dataTransfer.files;
    if (files.length > 0) {
        fileInput.files = files;
        handleFileSelect();
    }
}

function handleFileSelect() {
    const file = fileInput.files[0];
    if (!file) return;

    // Check if it's a WAV file
    if (!file.name.endsWith('.wav')) {
        showError('Please select a valid WAV file (.wav)');
        return;
    }

    uploadWavFile(file);
}

async function uploadWavFile(file) {
    // Reset UI
    hideError();
    analyzeBtn.disabled = true;
    saveBtn.classList.add('hidden');
    resultArea.classList.add('hidden');

    try {
        // Upload the file using the file manager
        const data = await fileManager.uploadFile(file);
        uploadedFilename = data.filename;

        uploadSuccess.textContent = `Successfully uploaded: ${file.name}`;
        uploadSuccess.classList.remove('hidden');
        analyzeBtn.disabled = false;
    } catch (error) {
        showError(error.message);
    }
}

// Debounced analysis function
function debounceAnalysis() {
    if (debounceTimeout) {
        clearTimeout(debounceTimeout);
    }
    debounceTimeout = setTimeout(() => {
        if (uploadedFilename) {
            analyzeWav(true);
        }
    }, 100); // 0.1s debounce
}

async function analyzeWav(isLiveUpdate = false) {
    if (!uploadedFilename) return;

    try {
        // Get parameters
        const samples = Math.pow(2, samplesSlider.value);
        const startTime = parseFloat(startTimeSlider.value);
        const baseFreq = parseInt(baseFreqSlider.value);
        const harmonics = parseInt(harmonicsSlider.value);

        // Call the harmonic-info endpoint
        const response = await fetch(`/harmonic-info/${uploadedFilename}?samples=${samples}&startTime=${startTime}&baseFreq=${baseFreq}&harmonics=${harmonics}`);

        if (!response.ok) {
            throw new Error(await response.text());
        }

        const data = await response.json();
        currentHarmonics = data.harmonics;

        // Display results
        resultArea.textContent = currentHarmonics.join(',');
        resultArea.classList.remove('hidden');
        saveBtn.classList.remove('hidden');
        previewBtn.classList.remove('hidden');
        volumeContainer.classList.remove('hidden');

        // Update the preview if it's playing
        if (isPlaying) {
            // Recreate the oscillator with new harmonics
            const wasPlaying = isPlaying;
            stopPreview();
            if (wasPlaying) {
                startPreview();
            }
        } else if (!isLiveUpdate) {
            // Start preview automatically on first analysis
            startPreview();
        }
    } catch (error) {
        showError(error.message);
    }
}

async function saveSoundfont() {
    if (!currentHarmonics) return;

    try {
        // Get a name for the soundfont
        const name = prompt('Enter a name for the soundfont:', 'custom');
        if (!name) return;

        // Save the soundfont
        const response = await fetch(`/save-soundfont/${name}`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(currentHarmonics)
        });

        if (!response.ok) {
            throw new Error(await response.text());
        }

        const data = await response.json();
        showSuccess(`Soundfont saved as: ${data.filename}`);
    } catch (error) {
        showError(error.message);
    }
}

// Parameter update functions
function updateSamplesValue() {
    const samples = Math.pow(2, samplesSlider.value);
    samplesValue.textContent = `${samples} samples`;
}

function updateStartTimeValue() {
    startTimeValue.textContent = `${startTimeSlider.value}s`;
}

function updateBaseFreqValue() {
    baseFreqValue.textContent = `${baseFreqSlider.value} Hz`;
}

function updateHarmonicsValue() {
    harmonicsValue.textContent = `${harmonicsSlider.value} harmonics`;
}

// Volume control
function updateVolume() {
    const volumePercent = volumeControl.value;
    volumeValue.textContent = `${volumePercent}%`;
    currentVolume = volumePercent / 100;

    if (gainNode) {
        gainNode.gain.value = currentVolume;
    }
}

// Helper functions
function showError(message) {
    uploadError.textContent = message;
    uploadError.classList.remove('hidden');
}

function showSuccess(message) {
    uploadSuccess.textContent = message;
    uploadSuccess.classList.remove('hidden');
}

function hideError() {
    uploadError.classList.add('hidden');
    uploadSuccess.classList.add('hidden');
}

// Audio preview functions
function initAudio() {
    if (!audioContext) {
        audioContext = new (window.AudioContext || window.webkitAudioContext)();
    }
}

function createOscillator() {
    if (!audioContext || !currentHarmonics) return;

    // Create nodes
    oscillator = audioContext.createOscillator();
    gainNode = audioContext.createGain();

    // Create periodic wave from harmonics
    const realCoef = new Float32Array(currentHarmonics.length + 1);
    const imagCoef = new Float32Array(currentHarmonics.length + 1);

    // DC offset should be 0
    realCoef[0] = 0;
    imagCoef[0] = 0;

    // Set harmonic coefficients
    for (let i = 0; i < currentHarmonics.length; i++) {
        realCoef[i + 1] = currentHarmonics[i];
        imagCoef[i + 1] = 0; // Using only cosine terms
    }

    const wave = audioContext.createPeriodicWave(realCoef, imagCoef, {
        disableNormalization: false
    });

    // Configure oscillator with fixed preview frequency
    oscillator.setPeriodicWave(wave);
    oscillator.frequency.value = PREVIEW_FREQUENCY;

    // Configure gain (volume)
    gainNode.gain.value = currentVolume;

    // Connect nodes
    oscillator.connect(gainNode);
    gainNode.connect(audioContext.destination);
}

function startPreview() {
    initAudio();
    createOscillator();

    if (oscillator) {
        oscillator.start();
        isPlaying = true;
        previewBtn.classList.add('playing');
        previewStatus.classList.remove('hidden');
    }
}

function stopPreview() {
    if (oscillator) {
        oscillator.stop();
        oscillator.disconnect();
        oscillator = null;
    }
    if (gainNode) {
        gainNode.disconnect();
        gainNode = null;
    }
    isPlaying = false;
    previewBtn.classList.remove('playing');
    previewStatus.classList.add('hidden');
}

function togglePreview() {
    if (isPlaying) {
        stopPreview();
    } else {
        startPreview();
    }
}

// Clean up audio context when leaving the page
window.addEventListener('beforeunload', () => {
    if (audioContext) {
        audioContext.close();
    }
});

// Initialize parameter values and handle any saved values
function initializeUI() {
    // Update display values
    updateSamplesValue();
    updateStartTimeValue();
    updateBaseFreqValue();
    updateHarmonicsValue();
    updateVolume();

    // If we have a saved volume value, update the internal state
    currentVolume = volumeControl.value / 100;
}

// Call initialization when the page loads
initializeUI();