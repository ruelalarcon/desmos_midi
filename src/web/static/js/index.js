// Global variables
let uploadedFilename = null;
let availableSoundfonts = [];
let channelInfo = [];
let fileExpirationMinutes = 10;
let fileRefreshThresholdMinutes = 5;
let fileRefreshIntervalId = null;

// DOM elements
const uploadArea = document.getElementById('upload-area');
const fileInput = document.getElementById('file-input');
const uploadError = document.getElementById('upload-error');
const uploadSuccess = document.getElementById('upload-success');
const uploadLoading = document.getElementById('upload-loading');
const step2 = document.getElementById('step-2');
const channelList = document.getElementById('channel-list');
const soundfontLoading = document.getElementById('soundfont-loading');
const convertBtn = document.getElementById('convert-btn');
const step3 = document.getElementById('step-3');
const resultArea = document.getElementById('result-area');
const convertLoading = document.getElementById('convert-loading');
const copyBtn = document.getElementById('copy-btn');
const copySuccess = document.getElementById('copy-success');

// Event listeners
uploadArea.addEventListener('click', () => fileInput.click());
uploadArea.addEventListener('dragover', handleDragOver);
uploadArea.addEventListener('dragleave', handleDragLeave);
uploadArea.addEventListener('drop', handleDrop);
fileInput.addEventListener('change', handleFileSelect);
convertBtn.addEventListener('click', convertMidi);
copyBtn.addEventListener('click', copyToClipboard);

// Initialize by loading available soundfonts
loadSoundfonts();

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

    // Check if it's a MIDI file
    if (!file.name.endsWith('.mid') && !file.name.endsWith('.midi')) {
        showError('Please select a valid MIDI file (.mid or .midi)');
        return;
    }

    uploadMidiFile(file);
}

function uploadMidiFile(file) {
    // Reset UI
    hideError();
    uploadLoading.classList.remove('hidden');

    // Stop any existing file refresh interval
    stopFileRefreshInterval();

    // Create form data
    const formData = new FormData();
    formData.append('midi_file', file);

    // Upload the file
    fetch('/upload', {
        method: 'POST',
        body: formData
    })
        .then(response => {
            if (!response.ok) {
                return response.text().then(text => {
                    throw new Error(text || 'Failed to upload MIDI file');
                });
            }
            return response.json();
        })
        .then(data => {
            uploadedFilename = data.filename;
            fileExpirationMinutes = data.expires_in_minutes || 10;
            fileRefreshThresholdMinutes = data.refresh_threshold_minutes || 5;

            uploadSuccess.textContent = `Successfully uploaded: ${file.name}`;
            uploadSuccess.classList.remove('hidden');

            // Start file refresh interval
            startFileRefreshInterval();

            // Load channel information
            loadChannelInfo(uploadedFilename);
        })
        .catch(error => {
            showError(error.message);
        })
        .finally(() => {
            uploadLoading.classList.add('hidden');
        });
}

function startFileRefreshInterval() {
    // Clear any existing interval
    stopFileRefreshInterval();

    if (!uploadedFilename) return;

    // Calculate refresh interval (check when 5 minutes are left)
    const checkIntervalMs = (fileExpirationMinutes - fileRefreshThresholdMinutes) * 60 * 1000;

    // Set up interval to refresh the file
    fileRefreshIntervalId = setInterval(() => {
        refreshFile(uploadedFilename);
    }, checkIntervalMs);

    console.log(`File refresh interval started. Will refresh every ${(fileExpirationMinutes - fileRefreshThresholdMinutes)} minutes.`);
}

function stopFileRefreshInterval() {
    if (fileRefreshIntervalId) {
        clearInterval(fileRefreshIntervalId);
        fileRefreshIntervalId = null;
    }
}

function refreshFile(filename) {
    if (!filename) return;

    console.log(`Refreshing file: ${filename}`);

    fetch('/refresh-file', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({
            filename: filename
        })
    })
        .then(response => {
            if (!response.ok) {
                console.error('Failed to refresh file');
                // If refresh fails, stop trying
                stopFileRefreshInterval();
                return;
            }
            return response.json();
        })
        .then(data => {
            console.log('File refreshed successfully', data);
        })
        .catch(error => {
            console.error('Error refreshing file:', error);
            // If refresh fails, stop trying
            stopFileRefreshInterval();
        });
}

function loadSoundfonts() {
    fetch('/soundfonts')
        .then(response => {
            if (!response.ok) {
                throw new Error('Failed to load soundfonts');
            }
            return response.json();
        })
        .then(data => {
            availableSoundfonts = data.soundfonts || [];
            // Add the ignore option
            availableSoundfonts.unshift('-');
        })
        .catch(error => {
            console.error('Error loading soundfonts:', error);
        });
}

function loadChannelInfo(filename) {
    // Reset UI
    channelList.innerHTML = '';
    soundfontLoading.classList.remove('hidden');
    step2.classList.remove('hidden');

    fetch(`/midi-info/${filename}`)
        .then(response => {
            if (!response.ok) {
                return response.text().then(text => {
                    throw new Error(text || 'Failed to load channel information');
                });
            }
            return response.json();
        })
        .then(data => {
            channelInfo = data.channels || [];
            populateChannelTable(channelInfo);
            convertBtn.disabled = false;
        })
        .catch(error => {
            showError(error.message);
        })
        .finally(() => {
            soundfontLoading.classList.add('hidden');
        });
}

function populateChannelTable(channels) {
    channelList.innerHTML = '';

    channels.forEach(channel => {
        const row = document.createElement('tr');

        // Channel number
        const channelCell = document.createElement('td');
        channelCell.textContent = channel.id;
        row.appendChild(channelCell);

        // Instrument name
        const instrumentCell = document.createElement('td');
        instrumentCell.textContent = channel.instrument;
        if (channel.is_drum) {
            instrumentCell.textContent += ' [DRUMS]';
        }
        row.appendChild(instrumentCell);

        // Soundfont selector
        const soundfontCell = document.createElement('td');
        const soundfontSelector = document.createElement('select');
        soundfontSelector.dataset.channel = channel.id;

        // Add options
        availableSoundfonts.forEach(soundfont => {
            const option = document.createElement('option');
            option.value = soundfont;
            option.textContent = soundfont;

            // Set default selection
            if (channel.is_drum && soundfont === '-') {
                option.selected = true;
            } else if (!channel.is_drum && soundfont === 'default.txt') {
                option.selected = true;
            }

            soundfontSelector.appendChild(option);
        });

        soundfontCell.appendChild(soundfontSelector);
        row.appendChild(soundfontCell);

        channelList.appendChild(row);
    });
}

function convertMidi() {
    // Get selected soundfonts
    const soundfonts = [];
    const selectors = document.querySelectorAll('select[data-channel]');

    selectors.forEach(selector => {
        soundfonts.push(selector.value);
    });

    // Reset UI
    convertLoading.classList.remove('hidden');
    step3.classList.remove('hidden');
    resultArea.textContent = '';
    copyBtn.disabled = true;

    // Send conversion request
    fetch('/convert', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({
            filename: uploadedFilename,
            soundfonts: soundfonts
        })
    })
        .then(response => {
            if (!response.ok) {
                return response.text().then(text => {
                    throw new Error(text || 'Failed to convert MIDI file');
                });
            }
            return response.json();
        })
        .then(data => {
            resultArea.textContent = data.formula;
            copyBtn.disabled = false;
        })
        .catch(error => {
            showError(error.message);
        })
        .finally(() => {
            convertLoading.classList.add('hidden');
        });
}

function copyToClipboard() {
    navigator.clipboard.writeText(resultArea.textContent)
        .then(() => {
            copySuccess.classList.remove('hidden');
            setTimeout(() => {
                copySuccess.classList.add('hidden');
            }, 3000);
        })
        .catch(err => {
            showError('Failed to copy to clipboard: ' + err.message);
        });
}

function showError(message) {
    uploadError.textContent = message;
    uploadError.classList.remove('hidden');
}

function hideError() {
    uploadError.classList.add('hidden');
    uploadSuccess.classList.add('hidden');
}

// Clean up when the page is unloaded
window.addEventListener('beforeunload', () => {
    stopFileRefreshInterval();
});