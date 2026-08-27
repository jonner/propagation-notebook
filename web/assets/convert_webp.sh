#!/bin/bash
# Convert image to WebP, resize to max width 1200px, quality 75%
input="$1"
output="${input%.*}.webp"

magick "$input" -resize 1200x1200\> -quality 75 "$output"
echo "Created WebP: $output"

