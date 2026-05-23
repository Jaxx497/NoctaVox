#!/usr/bin/env sh
# Download Noctavox theme examples directly from GitHub.
# Usage:  curl -fsSL https://raw.githubusercontent.com/Jaxx497/NoctaVox/master/get-themes.sh | sh
set -eu

OWNER="Jaxx497"
REPO="NoctaVox"
BRANCH="master"
THEME_DIR="${NOCTAVOX_THEME_DIR:-${XDG_CONFIG_HOME:-$HOME/.config}/noctavox/themes}"

API_URL="https://api.github.com/repos/$OWNER/$REPO/contents/docs/theme_examples?ref=$BRANCH"

if ! command -v curl >/dev/null 2>&1; then
    echo "Error: curl is required but not installed." >&2
    exit 1
fi

mkdir -p "$THEME_DIR"

urls=$(curl -fsSL -H "User-Agent: noctavox-installer" "$API_URL" \
    | grep -o '"download_url": *"[^"]*\.toml"' \
    | sed 's/.*"\(https[^"]*\)"/\1/')

if [ -z "$urls" ]; then
    echo "Error: failed to list themes from $API_URL" >&2
    echo "Check your network connection, or whether GitHub API rate limits apply." >&2
    exit 1
fi

count=0
echo "$urls" | while IFS= read -r url; do
    [ -z "$url" ] && continue
    name=$(basename "$url" | sed 's/%20/ /g')
    if curl -fsSL "$url" -o "$THEME_DIR/$name"; then
        echo "  $name"
    else
        echo "  ! failed: $name" >&2
    fi
done

total=$(echo "$urls" | grep -c .)
echo "Installed $total themes to $THEME_DIR"
