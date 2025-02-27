import * as fileManager from './modules/fileManager.js';

// DOM elements
const uploadArea = document.getElementById('upload-area');
const fileInput = document.getElementById('file-input');
const uploadError = document.getElementById('upload-error');
const uploadSuccess = document.getElementById('upload-success');
const analyzeBtn = document.getElementById('analyze-btn');
const resultArea = document.getElementById('result-area');
const saveBtn = document.getElementById('save-btn');

// Parameter elements
const samplesSlider = document.getElementById('samples');
const samplesValue = document.getElementById('samples-value');
const startTimeSlider = document.getElementById('start-time');
const startTimeValue = document.getElementById('start-time-value');
const baseFreqSlider = document.getElementById('base-freq');
const baseFreqValue = document.getElementById('base-freq-value');
const sensitivitySlider = document.getElementById('sensitivity');
const sensitivityValue = document.getElementById('sensitivity-value');

// State
let uploadedFilename = null;
let currentHarmonics = null;

// Event listeners
uploadArea.addEventListener('click', () => fileInput.click());
uploadArea.addEventListener('dragover', handleDragOver);
uploadArea.addEventListener('dragleave', handleDragLeave);
uploadArea.addEventListener('drop', handleDrop);
fileInput.addEventListener('change', handleFileSelect);
analyzeBtn.addEventListener('click', analyzeWav);
saveBtn.addEventListener('click', saveSoundfont);

// Parameter update listeners
samplesSlider.addEventListener('input', updateSamplesValue);
startTimeSlider.addEventListener('input', updateStartTimeValue);
baseFreqSlider.addEventListener('input', updateBaseFreqValue);
sensitivitySlider.addEventListener('input', updateSensitivityValue);

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

async function analyzeWav() {
    if (!uploadedFilename) return;

    try {
        // Get parameters
        const samples = Math.pow(2, samplesSlider.value);
        const startTime = parseFloat(startTimeSlider.value);
        const baseFreq = parseInt(baseFreqSlider.value);
        const sensitivity = parseInt(sensitivitySlider.value);

        // Call the harmonic-info endpoint
        const response = await fetch(`/harmonic-info/${uploadedFilename}?samples=${samples}&startTime=${startTime}&baseFreq=${baseFreq}&sensitivity=${sensitivity}`);

        if (!response.ok) {
            throw new Error(await response.text());
        }

        const data = await response.json();
        currentHarmonics = data.harmonics;

        // Display results
        resultArea.textContent = currentHarmonics.join(',');
        resultArea.classList.remove('hidden');
        saveBtn.classList.remove('hidden');
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

function updateSensitivityValue() {
    sensitivityValue.textContent = `${sensitivitySlider.value}%`;
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

// Initialize parameter values
updateSamplesValue();
updateStartTimeValue();
updateBaseFreqValue();
updateSensitivityValue(); 