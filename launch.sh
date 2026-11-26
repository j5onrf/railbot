#!/usr/bin/env bash
set -euo pipefail

APP_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DATA_DIR="${RAILBOT_HOME:-$HOME/.config/railbot}"
ICON_SRC="$APP_DIR/assets/railbot.svg"
[ -f "$ICON_SRC" ] || ICON_SRC="$DATA_DIR/assets/railbot.svg"

if [ "$(uname -s)" != "Darwin" ]; then
    DESKTOP_DIR="$HOME/.local/share/applications"
    ICON_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"
    mkdir -p "$DESKTOP_DIR" "$ICON_DIR"
    rm -f "$DESKTOP_DIR/rs-rail.desktop"
    [ -f "$ICON_SRC" ] && ln -sf "$ICON_SRC" "$ICON_DIR/railbot.svg"

    if [ ! -f "$DESKTOP_DIR/railbot.desktop" ] || ! grep -q "$APP_DIR/launch.sh" "$DESKTOP_DIR/railbot.desktop" 2>/dev/null; then
        cat <<DESKTOP > "$DESKTOP_DIR/railbot.desktop"
[Desktop Entry]
Version=1.0
Type=Application
Name=RailBot
GenericName=AI Agent Surface
Comment=Local-first desktop and web AI agent harness
Exec=$APP_DIR/launch.sh
Icon=railbot
Terminal=false
StartupNotify=true
Categories=Development;IDE;Utility;
Keywords=ai;agent;llm;rust;railbot;
DESKTOP
        update-desktop-database "$DESKTOP_DIR" >/dev/null 2>&1 || true
    fi
fi

BIN="${RAILBOT_BIN:-$APP_DIR/target/release/railbot}"
if [ ! -x "$BIN" ] && command -v railbot >/dev/null 2>&1; then
    BIN="$(command -v railbot)"
elif [ ! -x "$BIN" ] && [ -f "$APP_DIR/Cargo.toml" ]; then
    cargo build --release --manifest-path "$APP_DIR/Cargo.toml"
fi

PORT="${RAILBOT_PORT:-${RS_RAIL_PORT:-24243}}"
MODE="app"
TARGET_PATH="$PWD"

for arg in "$@"; do
    case "$arg" in
        web|--web) MODE="web" ;;
        *) [ -d "$arg" ] && TARGET_PATH="$arg" ;;
    esac
done

if ! curl -sf "http://127.0.0.1:$PORT/api/health" >/dev/null 2>&1; then
    nohup "$BIN" >"${XDG_RUNTIME_DIR:-/tmp}/railbot-$(id -u).log" 2>&1 &
    for _ in $(seq 1 20); do
        curl -sf "http://127.0.0.1:$PORT/api/health" >/dev/null 2>&1 && break
        sleep 0.1
    done
fi

PROJ=""
case "$TARGET_PATH" in
    "$DATA_DIR/projects/"*)
        PROJ="${TARGET_PATH#$DATA_DIR/projects/}"
        PROJ="${PROJ%%/*}"
        ;;
esac

URL="http://127.0.0.1:$PORT/${PROJ:+?project=$PROJ}"

if [ "$MODE" = "app" ]; then
    if [ "$(uname -s)" = "Darwin" ]; then
        for mac_browser in \
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
            "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser" \
            "/Applications/Chromium.app/Contents/MacOS/Chromium" \
            "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge"; do
            if [ -x "$mac_browser" ]; then
                nohup "$mac_browser" --app="$URL" --new-window >/dev/null 2>&1 </dev/null &
                exit 0
            fi
        done
    fi
    if command -v omarchy-launch-webapp >/dev/null 2>&1; then
        nohup setsid omarchy-launch-webapp "$URL" >/dev/null 2>&1 </dev/null &
        exit 0
    fi
    if command -v xdg-settings >/dev/null 2>&1; then
        DESKTOP=$(xdg-settings get default-web-browser 2>/dev/null || true)
        if [ -n "$DESKTOP" ]; then
            BIN_CMD=$(grep -hm1 '^Exec=' "$HOME/.local/share/applications/$DESKTOP" "/usr/share/applications/$DESKTOP" 2>/dev/null | head -n1 | sed 's/^Exec=//; s/ *%[a-zA-Z].*//; s/"//g' | awk '{print $1}' || true)
            case "${BIN_CMD##*/}" in
                *chrom*|*brave*|*vivaldi*|*edge*|*thorium*|*helium*|*opera*)
                    if command -v "$BIN_CMD" >/dev/null 2>&1; then
                        nohup setsid "$BIN_CMD" --app="$URL" --new-window >/dev/null 2>&1 </dev/null &
                        exit 0
                    fi
                    ;;
            esac
        fi
    fi
    for browser in chromium chromium-browser google-chrome google-chrome-stable brave brave-bin brave-browser brave-origin brave-origin-bin vivaldi vivaldi-stable microsoft-edge thorium-browser ungoogled-chromium; do
        if command -v "$browser" >/dev/null 2>&1; then
            nohup setsid "$browser" --app="$URL" --new-window >/dev/null 2>&1 </dev/null &
            exit 0
        fi
    done
fi

if command -v xdg-open >/dev/null 2>&1; then
    nohup xdg-open "$URL" >/dev/null 2>&1 </dev/null &
elif command -v open >/dev/null 2>&1; then
    open "$URL"
else
    echo "Open: $URL"
fi
