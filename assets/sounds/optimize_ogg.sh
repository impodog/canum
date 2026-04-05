#!/bin/bash
InputFile="$1"
ffmpeg -i "$InputFile" -c:a libopus -b:a "48k" -vbr on -y output.ogg
