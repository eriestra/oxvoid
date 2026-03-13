#!/bin/sh
# Set the publish secret for ox∅
set -e

SECRET="$1"
if [ -z "$SECRET" ]; then
  SECRET=$(openssl rand -base64 24)
  echo "generated secret: $SECRET"
fi

npx convex run seed:seedSecret "{\"secret\":\"$SECRET\"}"

# Append to .env.local if not already there
if ! grep -q '^PUBLISH_SECRET=' .env.local 2>/dev/null; then
  echo "PUBLISH_SECRET=$SECRET" >> .env.local
  echo "saved to .env.local"
else
  sed -i '' "s|^PUBLISH_SECRET=.*|PUBLISH_SECRET=$SECRET|" .env.local
  echo "updated .env.local"
fi
