#!/usr/bin/env sh

THEME_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/noctavox/themes"

mkdir -p "$THEME_DIR"
cp docs/theme_examples/*.toml "$THEME_DIR"

echo "Installed themes to $THEME_DIR"
