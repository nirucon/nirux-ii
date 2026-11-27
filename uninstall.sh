#!/usr/bin/env bash
# Remove NIRUX II: the program, its desktop entry and icon.
#
#   ./uninstall.sh           keep settings, progress and high scores
#   ./uninstall.sh --purge   also remove ~/.config/nirux-ii and the
#                            saved progress and high scores
set -euo pipefail
DATA="${XDG_DATA_HOME:-$HOME/.local/share}"
CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}"
APPS="$DATA/applications"

rm -f "$HOME/.local/bin/nirux-ii" "$APPS/nirux-ii.desktop" "$DATA/icons/hicolor/scalable/apps/nirux-ii.svg"
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$APPS" >/dev/null 2>&1 || true

if [[ ${1:-} == --purge ]]; then
  rm -rf "$CONFIG/nirux-ii" "$DATA/nirux-ii"
  printf "Removed NIRUX, its settings, progress and high scores.\n"
else
  printf "Removed NIRUX. Settings, progress and high scores were kept (--purge removes them).\n"
fi
