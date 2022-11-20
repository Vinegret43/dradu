#!/bin/sh

# === How it works ===
# This thing takes your svg icons and converts them into RGBA bytes. Also, for
# images in ./source/light directory, it creates a copy with colors which are
# negated (For dark theme). Also outputs Rust code to stdout so you
# can easily add these images to the client app

# === How to use ===
# 0. Install Inkscape and ImageMagick if you don't have them
# 1. Throw your light theme svg icons into ./source/light
# 2. Throw icons that are compatible with both light/dark themes into
#    ./source/general
# 3. Run `./convert.sh`
# 4. Copy/paste generated output into textures.rs
# 5. Run `cp -r ./rgba/* <path_to_client>/assets/img/`

mkdir .png 2> /dev/null
mkdir .png/general .png/light 2> /dev/null
mkdir rgba 2> /dev/null
mkdir rgba/dark rgba/light rgba/general 2> /dev/null

log() {
  echo -e "\033[33m$@\033[0m" >&2
}

log "Converting images in ./source/general. Copy/paste this into load_textures():"

for file in $(ls "./source/general"); do
  name=${file%.*}
  inkscape "./source/general/$file" -o "./png/light/name.png" &> /dev/null
  convert "./.png/general/$name.png" -depth 8 "./rgba/$name.rgba"
  resolution=$(identify "./source/general/$file" | grep -o '\w*x\w* ' | sed 's/ //;s/x/, /')
  echo "add_texture!(ctx, map, \"$name\", [$resolution]);"
done

log "\nConverting images in ./source/light. Copy/paste this into \
load_themed_textures():"

for file in $(ls "./source/light"); do
  name=${file%.*}
  inkscape "./source/light/$file" -o "./.png/light/$name.png" &> /dev/null
  convert "./.png/light/$name.png" -depth 8 "./rgba/light/$name.rgba"
  convert "./.png/light/$name.png" -channel RGB -negate -depth 8 "./rgba/dark/$name.rgba"
  resolution=$(identify "./source/light/$file" | grep -o '\w*x\w* ' | sed 's/ //;s/x/, /')
  echo "add_themed_texture!(ctx, maps, \"$name\", [$resolution]);"
done

