#!/bin/sh
# ox∅ publish — build WASM + upload to Convex → live URL
set -e

SLUG="$1"
if [ -z "$SLUG" ]; then
  echo "Usage: sh publish.sh <slug>"
  exit 1
fi

PUBLISH_SECRET=$(grep '^PUBLISH_SECRET=' .env.local | cut -d= -f2)
CONVEX_SITE_URL=$(grep '^CONVEX_SITE_URL=' .env.local | cut -d= -f2)

if [ -z "$PUBLISH_SECRET" ]; then
  echo "error: no PUBLISH_SECRET in .env.local"
  echo "run: sh seed.sh <your-secret>"
  exit 1
fi

echo "ox∅ publish: $SLUG"

# 1. Build WASM
echo "  build..."
sh build.sh

# 2. Upload WASM binary (base64-encoded)
echo "  upload wasm..."
WASM_B64=$(base64 < pkg/oxvoid_bg.wasm)
npx convex run assets:set "{\"name\":\"wasm/oxvoid_bg.wasm\",\"content\":\"$WASM_B64\",\"contentType\":\"application/wasm\",\"secret\":\"$PUBLISH_SECRET\"}" 2>/dev/null

# 3. Upload JS glue
echo "  upload js..."
JS_CONTENT=$(cat pkg/oxvoid.js)
npx convex run assets:set "{\"name\":\"js/oxvoid.js\",\"content\":$(echo "$JS_CONTENT" | python3 -c 'import sys,json; print(json.dumps(sys.stdin.read()))'),\"contentType\":\"application/javascript\",\"secret\":\"$PUBLISH_SECRET\"}" 2>/dev/null

# 4. Upload ox.css
echo "  upload css..."
CSS_CONTENT=$(cat ox.css)
npx convex run assets:set "{\"name\":\"css/ox\",\"content\":$(echo "$CSS_CONTENT" | python3 -c 'import sys,json; print(json.dumps(sys.stdin.read()))'),\"contentType\":\"text/css\",\"secret\":\"$PUBLISH_SECRET\"}" 2>/dev/null

# 5. Build HTML shell that references Convex-hosted assets
echo "  publish page..."
HTML=$(cat <<HTMLEOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>$SLUG — ox∅</title>
    <link rel="preconnect" href="https://fonts.googleapis.com" />
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
    <link href="https://fonts.googleapis.com/css2?family=DM+Sans:wght@300;400;500;600;700&family=JetBrains+Mono:wght@400;500;600;700&family=Outfit:wght@400;500;600;700;800;900&display=swap" rel="stylesheet" />
    <link rel="stylesheet" href="/css/ox" />
</head>
<body>
    <div id="app"></div>
    <script type="module">
        import init from '/js/oxvoid.js';
        init('/wasm/oxvoid_bg.wasm');
    </script>
</body>
</html>
HTMLEOF
)

npx convex run pages:publish "{\"slug\":\"$SLUG\",\"html\":$(echo "$HTML" | python3 -c 'import sys,json; print(json.dumps(sys.stdin.read()))'),\"secret\":\"$PUBLISH_SECRET\"}" 2>/dev/null

echo ""
echo "  live: $CONVEX_SITE_URL/app/$SLUG"
echo "  done."
