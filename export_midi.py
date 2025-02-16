import json
import os
import sys
from collections import defaultdict

import mido


def midi_note_to_relative(note):
    """Convert MIDI note number to relative position from middle C (60)"""
    return note - 60

def create_export_folder(midi_path):
    """Create folder based on MIDI filename"""
    folder_name = os.path.splitext(os.path.basename(midi_path))[0]
    os.makedirs(folder_name, exist_ok=True)
    return folder_name

def process_midi(midi_path):
    # Parse MIDI file
    mid = mido.MidiFile(midi_path)

    # Track active notes and their changes
    active_notes = set()
    note_changes = defaultdict(set)

    # Keep track of current time in seconds
    current_time = 0

    # Process all messages in all tracks
    for msg in mid:
        # Update current time
        current_time += msg.time
        current_time_ms = int(current_time * 1000)  # Convert to milliseconds

        if msg.type == 'note_on' and msg.velocity > 0:
            active_notes.add(msg.note)
            note_changes[current_time_ms] = set(active_notes)
        elif msg.type == 'note_off' or (msg.type == 'note_on' and msg.velocity == 0):
            if msg.note in active_notes:
                active_notes.remove(msg.note)
                note_changes[current_time_ms] = set(active_notes)

    # Create folder
    folder_name = create_export_folder(midi_path)

    # Export data.json
    data_json = {
        str(time): sorted(list(notes))
        for time, notes in note_changes.items()
    }

    with open(f"{folder_name}/data.json", 'w') as f:
        json.dump(data_json, f, indent=2)

    # Export formulas.txt
    formulas = []
    for notes in note_changes.values():
        relative_notes = [midi_note_to_relative(note) for note in sorted(notes)]
        formula = f"A\\to\\left[{','.join(map(str, relative_notes))}\\right]"
        formulas.append(formula)

    with open(f"{folder_name}/formulas.txt", 'w') as f:
        f.write('\n'.join(formulas))

def main():
    if len(sys.argv) != 2:
        print("Usage: python export_midi.py <midi_file>")
        sys.exit(1)

    midi_path = sys.argv[1]
    if not os.path.exists(midi_path):
        print(f"Error: File {midi_path} not found")
        sys.exit(1)

    process_midi(midi_path)

if __name__ == "__main__":
    main()
