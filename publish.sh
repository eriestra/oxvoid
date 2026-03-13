#!/bin/sh
set -e

SLUG="$1"
if [ -z "$SLUG" ]; then
  echo "Usage: sh publish.sh <slug>"
  exit 1
fi

echo "ox∅ publish: $SLUG"
sh build.sh
PUBLISH_SECRET=$(grep '^PUBLISH_SECRET=' .env.local | cut -d= -f2)
# TODO: upload dist/ + index.html + ox.css to Convex pages table
echo "done: $SLUG"
