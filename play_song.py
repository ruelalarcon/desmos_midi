import json
import os
import sys
import time
from threading import Event

import keyboard
import tomli


def load_config():
    """Load configuration from config.toml"""
    with open('config.toml', 'rb') as f:
        return tomli.load(f)

def load_song_data(folder_name):
    """Load the song data from data.json"""
    with open(os.path.join(folder_name, 'data.json'), 'r') as f:
        return json.load(f)

def format_time(ms):
    """Convert milliseconds to MM:SS format"""
    total_seconds = int(ms / 1000)
    minutes = total_seconds // 60
    seconds = total_seconds % 60
    return f"{minutes:02d}:{seconds:02d}"

def play_song(folder_name):
    # Load config and song data
    config = load_config()
    song_data = load_song_data(folder_name)

    # Convert timestamps to integers and sort them
    timestamps = sorted(int(t) for t in song_data.keys())
    total_duration = timestamps[-1]

    # Create stop event
    stop_event = Event()
    keyboard.on_press_key(config['keyboard']['stop_key'], lambda _: stop_event.set())

    print(f"Playing {folder_name} in {config['timing']['initial_delay']} seconds...")
    print(f"Press '{config['keyboard']['stop_key']}' to stop playback")
    print(f"Total duration: {format_time(total_duration)}")
    time.sleep(config['timing']['initial_delay'])

    # Simulate initial key sequence to start playing
    for key in config['keyboard']['start_sequence']:
        keyboard.send(key)

    # Record start time
    start_time = time.time() * 1000  # Convert to milliseconds

    try:
        # Play through each timestamp
        for i in range(len(timestamps) - 1):
            if stop_event.is_set():
                print("\nPlayback stopped")
                return

            # Calculate how long to wait
            current_time = time.time() * 1000
            elapsed = current_time - start_time
            next_timestamp = timestamps[i + 1]

            # Show progress
            print(f"\rProgress: {format_time(elapsed)} / {format_time(total_duration)}", end="", flush=True)

            # Wait until it's time for the next note
            wait_time = (next_timestamp - elapsed) / 1000  # Convert back to seconds
            if wait_time > 0:
                time.sleep(wait_time)

            # Add configured delay
            if config['timing']['note_delay_ms'] > 0:
                time.sleep(config['timing']['note_delay_ms'] / 1000)

            # Simulate configured key sequence to move to next note
            for key in config['keyboard']['next_sequence']:
                keyboard.send(key)

        # Show final progress
        print(f"\rProgress: {format_time(total_duration)} / {format_time(total_duration)}")

    except KeyboardInterrupt:
        print("\nPlayback stopped")
    finally:
        keyboard.unhook_all()

def main():
    if len(sys.argv) != 2:
        print("Usage: python play_song.py <song_folder>")
        sys.exit(1)

    folder_name = sys.argv[1]
    if not os.path.exists(folder_name):
        print(f"Error: Folder {folder_name} not found")
        sys.exit(1)

    if not os.path.exists(os.path.join(folder_name, 'data.json')):
        print(f"Error: data.json not found in {folder_name}")
        sys.exit(1)

    play_song(folder_name)

if __name__ == "__main__":
    main()