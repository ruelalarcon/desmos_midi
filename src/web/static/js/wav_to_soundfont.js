import * as FileManager from './modules/FileManager.js';

// DOM elements
const uploadArea = document.getElementById('upload-area');
const fileInput = document.getElementById('file-input');
const uploadError = document.getElementById('upload-error');
const uploadSuccess = document.getElementById('upload-success');
const uploadLoading = document.getElementById('upload-loading');
const parametersSection = document.getElementById('parameters-section');
const resultSection = document.getElementById('result-section');
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

// Parameter update functions
function updateSamplesValue() {
    const samples = Math.pow(2, samplesSlider.value);
    samplesValue.textContent = `${samples} samples`;
}

function updateStartTimeValue() {
    startTimeValue.value = startTimeSlider.value;
}

function updateBaseFreqValue() {
    baseFreqValue.value = baseFreqSlider.value;
}

function updateHarmonicsValue() {
    harmonicsValue.value = harmonicsSlider.value;
}

function handleStartTimeInput(event) {
    let value = parseFloat(event.target.value);
    if (isNaN(value)) {
        value = parseFloat(startTimeSlider.value);
    } else {
        // Clamp value between min and max
        value = Math.max(startTimeSlider.min, Math.min(startTimeSlider.max, value));
    }
    startTimeSlider.value = value;
    updateStartTimeValue();
    debounceAnalysis();
}

function handleBaseFreqInput(event) {
    let value = parseInt(event.target.value);
    if (isNaN(value)) {
        value = parseInt(baseFreqSlider.value);
    } else {
        // Clamp value between min and max
        value = Math.max(baseFreqSlider.min, Math.min(baseFreqSlider.max, value));
    }
    baseFreqSlider.value = value;
    updateBaseFreqValue();
    debounceAnalysis();
}

function handleHarmonicsInput(event) {
    let value = parseInt(event.target.value);
    if (isNaN(value)) {
        value = parseInt(harmonicsSlider.value);
    } else {
        // Clamp value between min and max
        value = Math.max(harmonicsSlider.min, Math.min(harmonicsSlider.max, value));
    }
    harmonicsSlider.value = value;
    updateHarmonicsValue();
    debounceAnalysis();
}

// Add event listeners for input fields
startTimeValue.addEventListener('blur', handleStartTimeInput);
startTimeValue.addEventListener('keypress', (event) => {
    if (event.key === 'Enter') {
        event.preventDefault();
        handleStartTimeInput(event);
        startTimeValue.blur();
    }
});

baseFreqValue.addEventListener('blur', handleBaseFreqInput);
baseFreqValue.addEventListener('keypress', (event) => {
    if (event.key === 'Enter') {
        event.preventDefault();
        handleBaseFreqInput(event);
        baseFreqValue.blur();
    }
});

harmonicsValue.addEventListener('blur', handleHarmonicsInput);
harmonicsValue.addEventListener('keypress', (event) => {
    if (event.key === 'Enter') {
        event.preventDefault();
        handleHarmonicsInput(event);
        harmonicsValue.blur();
    }
});

// Volume control
function updateVolume() {
    const volumePercent = volumeControl.value;
    volumeValue.textContent = `${volumePercent}%`;
    currentVolume = volumePercent / 100;

    if (gainNode) {
        gainNode.gain.value = currentVolume;
    }
}

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
    uploadLoading.classList.remove('hidden');
    resultSection.classList.add('hidden');
    previewBtn.classList.add('hidden');
    volumeContainer.classList.add('hidden');
    saveBtn.classList.add('hidden');

    try {
        // Upload the file using the file manager
        const data = await FileManager.uploadFile(file);
        uploadedFilename = data.filename;

        uploadSuccess.textContent = `Successfully uploaded: ${file.name}`;
        uploadSuccess.classList.remove('hidden');
        
        // Show parameters section and analyze immediately
        parametersSection.classList.remove('hidden');
        analyzeWav(false);
    } catch (error) {
        showError(error.message);
    } finally {
        uploadLoading.classList.add('hidden');
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
        hideMessages(); // Clear any existing messages

        // Get parameters
        const samples = Math.pow(2, samplesSlider.value);
        const startTime = parseFloat(startTimeSlider.value);
        const baseFreq = parseInt(baseFreqSlider.value);
        const harmonics = parseInt(harmonicsSlider.value);

        // Call the harmonic-info endpoint
        const response = await fetch(`/harmonic-info/${uploadedFilename}?samples=${samples}&startTime=${startTime}&baseFreq=${baseFreq}&harmonics=${harmonics}`);

        const text = await response.text();
        if (!response.ok) {
            throw new Error(text);
        }

        const data = JSON.parse(text);
        currentHarmonics = data.harmonics;

        // Display results
        resultArea.textContent = currentHarmonics.join(',');
        resultSection.classList.remove('hidden');
        previewBtn.classList.remove('hidden');
        volumeContainer.classList.remove('hidden');
        saveBtn.classList.remove('hidden');

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
        hideMessages(); // Clear any existing messages

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

        const text = await response.text();
        if (!response.ok) {
            throw new Error(text);
        }

        const data = JSON.parse(text);
        showSuccess(`Soundfont saved as: ${data.filename}`);
    } catch (error) {
        showError(error.message);
    }
}

// Helper functions
function showError(message) {
    const errorElement = document.getElementById('analysis-error');
    errorElement.textContent = message;
    errorElement.classList.remove('hidden');
    document.getElementById('analysis-success').classList.add('hidden');
}

function showSuccess(message) {
    const successElement = document.getElementById('analysis-success');
    successElement.textContent = message;
    successElement.classList.remove('hidden');
    document.getElementById('analysis-error').classList.add('hidden');
}

function hideError() {
    document.getElementById('analysis-error').classList.add('hidden');
}

function hideSuccess() {
    document.getElementById('analysis-success').classList.add('hidden');
}

function hideMessages() {
    hideError();
    hideSuccess();
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

    // Add click handler for preview button
    previewBtn.addEventListener('click', togglePreview);
    saveBtn.addEventListener('click', saveSoundfont);
}

// Call initialization when the page loads
initializeUI();