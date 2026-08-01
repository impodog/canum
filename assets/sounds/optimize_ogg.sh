#!/usr/bin/env bash
set -euo pipefail

# --- Configuration & Checks ---
if ! command -v ffmpeg &> /dev/null; then
    echo "❌ Error: ffmpeg is not installed. Please install it first." >&2
    exit 1
fi

if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <input_audio_file>" >&2
    exit 1
fi

INPUT_FILE="$1"

if [[ ! -f "$INPUT_FILE" ]]; then
    echo "❌ Error: File '$INPUT_FILE' not found." >&2
    exit 1
fi

# Derive output filename by replacing the extension with .ogg
OUTPUT_FILE="${INPUT_FILE%.*}.ogg"

if [[ "$INPUT_FILE" == "$OUTPUT_FILE" ]] then
    OUTPUT_FILE="$2"
fi

echo "🎵 Converting: $INPUT_FILE -> $OUTPUT_FILE"
echo "   Format : OGG (Vorbis)"
echo "   Bitrate: 48 kbps"
echo "   Metadata & Images: Stripped"
echo ""

# --- Conversion ---
if ffmpeg -i "$INPUT_FILE" \
           -map 0:a:0 \
           -map_metadata -1 \
           -c:a libvorbis \
           -b:a 48k \
           -y \
           "$OUTPUT_FILE" 2>/dev/null; then
    echo "✅ Success: '$OUTPUT_FILE' created."
else
    echo "❌ Conversion failed. Check if the input file is valid." >&2
    exit 1
fi
