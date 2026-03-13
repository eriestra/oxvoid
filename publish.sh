#!/bin/sh
# ox∅ publish — build WASM + upload to Convex → live URL
set -e

SLUG="$1"
if [ -z "$SLUG" ]; then
  echo "Usage: sh publish.sh <slug>"
  exit 1
fi

# Load env
PUBLISH_SECRET=$(grep '^PUBLISH_SECRET=' .env.local | cut -d= -f2)
CONVEX_URL=$(grep '^CONVEX_URL=' .env.local | cut -d= -f2)
CONVEX_SITE_URL=$(grep '^CONVEX_SITE_URL=' .env.local | cut -d= -f2)

if [ -z "$PUBLISH_SECRET" ]; then
  echo "error: no PUBLISH_SECRET in .env.local — run: sh seed.sh"
  exit 1
fi

echo "ox∅ publish: $SLUG"

# 1. Build WASM
echo "  build..."
sh build.sh

# 2. Upload via Convex client (no shell escaping issues)
PUBLISH_SECRET="$PUBLISH_SECRET" CONVEX_URL="$CONVEX_URL" node upload.mjs "$SLUG"

echo "  live: $CONVEX_SITE_URL/app/$SLUG"
