#!/bin/bash
# Convert image to WebP, resize to max width 1200px, quality 75%
input="$1"
output="${input%.*}.webp"

magick "$input" -resize 1200x1200 -strip -define webp:method=6 -define webp:alpha-compression=1 -define webp:auto-filter=true -quality 65 "$output"
echo "Created WebP: $output"

