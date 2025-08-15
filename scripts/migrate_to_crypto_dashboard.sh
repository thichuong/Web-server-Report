#!/usr/bin/env bash
set -euo pipefail

# Migration script: move templates/ and static/ into crypto_dashboard/
# Run from repository root.

ROOT_DIR="$(pwd)"
echo "Running migration from $ROOT_DIR"

# Create target structure
mkdir -p crypto_dashboard/pages
mkdir -p crypto_dashboard/templates
mkdir -p crypto_dashboard/assets/js/pages
mkdir -p crypto_dashboard/assets/js/modules
mkdir -p crypto_dashboard/assets/js/chart_modules
mkdir -p crypto_dashboard/assets/css/shared
mkdir -p crypto_dashboard/assets/css/charts
mkdir -p crypto_dashboard/assets

# Move templates (preserve tree)
if [ -d templates ]; then
  echo "Moving templates/ -> crypto_dashboard/templates/"
  git mv templates crypto_dashboard/templates || {
    # fallback: move contents
    mv templates/* crypto_dashboard/templates/ 2>/dev/null || true
    git add crypto_dashboard/templates || true
  }
else
  echo "No templates/ dir found, skipping"
fi

# Move top-level static HTML files into pages
shopt -s nullglob
for f in static/*.html; do
  echo "Moving $f -> crypto_dashboard/pages/"
  git mv "$f" crypto_dashboard/pages/ || mv "$f" crypto_dashboard/pages/ || true
done

# Move manifest and service worker
for f in static/*.json static/sw.js static/sw.js; do
  if [ -f "$f" ]; then
    echo "Moving $f -> crypto_dashboard/assets/"
    git mv "$f" crypto_dashboard/assets/ || mv "$f" crypto_dashboard/assets/ || true
  fi
done

# Move static/js (preserve modules and chart_modules)
if [ -d static/js ]; then
  echo "Moving static/js -> crypto_dashboard/assets/js/"
  git mv static/js crypto_dashboard/assets/js || mv static/js crypto_dashboard/assets/js || true
else
  echo "No static/js found, skipping"
fi

# Move static/css
if [ -d static/css ]; then
  echo "Moving static/css -> crypto_dashboard/assets/css/"
  git mv static/css crypto_dashboard/assets/css || mv static/css crypto_dashboard/assets/css || true
else
  echo "No static/css found, skipping"
fi

# If there are remaining files directly under static (like manifest.json), move them
for f in static/*; do
  # skip dirs
  if [ -f "$f" ]; then
    # skip html/js/css/json already moved
    case "$f" in
      *.html|*.js|*.css|*.json) continue ;;
    esac
    echo "Moving leftover $f -> crypto_dashboard/assets/"
    git mv "$f" crypto_dashboard/assets/ || mv "$f" crypto_dashboard/assets/ || true
  fi
done

# Commit changes if any
if git status --porcelain | grep . >/dev/null 2>&1; then
  git add -A
  git commit -m "chore: move templates/ and static/ into crypto_dashboard/ (initial reorganize)"
  echo "Committed changes"
else
  echo "No changes to commit"
fi

echo "Migration finished. Please inspect crypto_dashboard/ and update any hard-coded paths in templates or JS if needed."
