/**
 * File Manager Module
 * Handles all file operations including uploading, retrieving,
 * getting MIDI info, and refreshing files to prevent deletion
 */

// Configuration
const config = {
    fileExpirationMinutes: 10,
    fileRefreshThresholdMinutes: 5
};

// State
let activeFiles = new Map(); // filename -> {refreshInterval, expiresAt}

/**
 * Upload a MIDI file to the server
 * @param {File} file - The file object to upload
 * @returns {Promise<Object>} - Promise resolving to {filename, expiresInMinutes}
 */
export async function uploadFile(file) {
    // Create form data
    const formData = new FormData();
    formData.append('midi_file', file);

    // Upload the file
    const response = await fetch('/upload', {
        method: 'POST',
        body: formData
    });

    if (!response.ok) {
        const errorText = await response.text();
        throw new Error(errorText || 'Failed to upload file');
    }

    const data = await response.json();

    // Store file info and start refresh interval
    startFileRefreshInterval(data.filename, data.expires_in_minutes || config.fileExpirationMinutes);

    return data;
}

/**
 * Get MIDI file information
 * @param {string} filename - The filename to get info for
 * @returns {Promise<Object>} - Promise resolving to the MIDI info
 */
export async function getMidiInfo(filename) {
    const response = await fetch(`/midi-info/${filename}`);

    if (!response.ok) {
        const errorText = await response.text();
        throw new Error(errorText || 'Failed to load MIDI information');
    }

    return await response.json();
}

/**
 * Convert MIDI file with selected soundfonts
 * @param {string} filename - The filename to convert
 * @param {Array<string>} soundfonts - Array of soundfont selections
 * @returns {Promise<Object>} - Promise resolving to {formula}
 */
export async function convertMidi(filename, soundfonts) {
    const response = await fetch('/convert', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({
            filename: filename,
            soundfonts: soundfonts
        })
    });

    if (!response.ok) {
        const errorText = await response.text();
        throw new Error(errorText || 'Failed to convert MIDI file');
    }

    return await response.json();
}

/**
 * Get a file from the server
 * @param {string} filename - The filename to retrieve
 * @returns {Promise<Blob>} - Promise resolving to the file blob
 */
export async function getFile(filename) {
    const response = await fetch(`/getfile/${filename}`);

    if (!response.ok) {
        const errorText = await response.text();
        throw new Error(errorText || 'Failed to retrieve file');
    }

    return await response.blob();
}

/**
 * Get available soundfonts
 * @returns {Promise<Array<string>>} - Promise resolving to array of soundfont names
 */
export async function getSoundfonts() {
    const response = await fetch('/soundfonts');

    if (!response.ok) {
        throw new Error('Failed to load soundfonts');
    }

    const data = await response.json();
    return data.soundfonts || [];
}

/**
 * Refresh a file to prevent it from being deleted
 * @param {string} filename - The filename to refresh
 * @returns {Promise<Object>} - Promise resolving to the refresh response
 */
export async function refreshFile(filename) {
    if (!filename) return null;

    console.log(`Refreshing file: ${filename}`);

    try {
        const response = await fetch('/refresh-file', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                filename: filename
            })
        });

        if (!response.ok) {
            console.error('Failed to refresh file');
            stopFileRefreshInterval(filename);
            return null;
        }

        const data = await response.json();
        console.log('File refreshed successfully', data);
        return data;
    } catch (error) {
        console.error('Error refreshing file:', error);
        stopFileRefreshInterval(filename);
        return null;
    }
}

/**
 * Start an interval to refresh a file before it expires
 * @param {string} filename - The filename to refresh
 * @param {number} expirationMinutes - Minutes until the file expires
 */
function startFileRefreshInterval(filename, expirationMinutes) {
    // Clear any existing interval for this file
    stopFileRefreshInterval(filename);

    if (!filename) return;

    // Calculate when the file expires
    const expiresAt = new Date(Date.now() + (expirationMinutes * 60 * 1000));

    // Calculate refresh interval (check when threshold minutes are left)
    const refreshThresholdMs = config.fileRefreshThresholdMinutes * 60 * 1000;
    const checkIntervalMs = Math.max(
        (expirationMinutes - config.fileRefreshThresholdMinutes) * 60 * 1000,
        60000 // At least check every minute
    );

    // Set up interval to refresh the file
    const intervalId = setInterval(() => {
        const timeLeft = expiresAt - Date.now();

        // If less than threshold time left, refresh the file
        if (timeLeft <= refreshThresholdMs) {
            refreshFile(filename);
        }
    }, checkIntervalMs);

    // Store the interval ID and expiration time
    activeFiles.set(filename, {
        refreshInterval: intervalId,
        expiresAt: expiresAt
    });

    console.log(`File refresh interval started for ${filename}. Will refresh when ${config.fileRefreshThresholdMinutes} minutes are left before expiration.`);
}

/**
 * Stop the refresh interval for a file
 * @param {string} filename - The filename to stop refreshing
 */
function stopFileRefreshInterval(filename) {
    const fileInfo = activeFiles.get(filename);
    if (fileInfo && fileInfo.refreshInterval) {
        clearInterval(fileInfo.refreshInterval);
        activeFiles.delete(filename);
        console.log(`File refresh interval stopped for ${filename}`);
    }
}

/**
 * Stop all file refresh intervals
 * Called when the page is unloaded
 */
export function cleanup() {
    for (const [filename, fileInfo] of activeFiles.entries()) {
        if (fileInfo.refreshInterval) {
            clearInterval(fileInfo.refreshInterval);
            console.log(`File refresh interval stopped for ${filename}`);
        }
    }
    activeFiles.clear();
}

// Set up cleanup when the page is unloaded
window.addEventListener('beforeunload', cleanup);