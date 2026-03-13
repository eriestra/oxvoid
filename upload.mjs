// ox∅ asset uploader — uses Convex HTTP client directly
import { readFileSync } from "fs";

const secret = process.env.PUBLISH_SECRET;
const slug = process.argv[2];
const convexUrl = process.env.CONVEX_URL;

if (!secret || !slug || !convexUrl) {
  console.error("Missing PUBLISH_SECRET, CONVEX_URL, or slug");
  process.exit(1);
}

async function mutate(path, args) {
  const res = await fetch(`${convexUrl}/api/mutation`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ path, args }),
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`${path} failed: ${res.status} ${text}`);
  }
  return res.json();
}

// Upload WASM (base64) — per-slug to avoid collisions
console.log("  upload wasm...");
const wasmB64 = readFileSync("pkg/oxvoid_bg.wasm").toString("base64");
await mutate("assets:set", {
  name: `wasm/${slug}_bg.wasm`,
  content: wasmB64,
  contentType: "application/wasm",
  secret,
});

// Upload JS glue — patch it to reference the slug-specific WASM
console.log("  upload js...");
let jsContent = readFileSync("pkg/oxvoid.js", "utf-8");
jsContent = jsContent.replaceAll("oxvoid_bg.wasm", `${slug}_bg.wasm`);
await mutate("assets:set", {
  name: `js/${slug}.js`,
  content: jsContent,
  contentType: "application/javascript",
  secret,
});

// Upload ox.css
console.log("  upload css...");
const cssContent = readFileSync("ox.css", "utf-8");
await mutate("assets:set", {
  name: "css/ox",
  content: cssContent,
  contentType: "text/css",
  secret,
});

// Publish HTML shell
console.log("  publish page...");
const html = `<!DOCTYPE html>
<html lang="en" data-theme="dark">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>${slug} — ox∅</title>
    <link rel="preconnect" href="https://fonts.googleapis.com" />
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
    <link href="https://fonts.googleapis.com/css2?family=DM+Sans:wght@300;400;500;600;700&family=JetBrains+Mono:wght@400;500;600;700&family=Outfit:wght@400;500;600;700;800;900&display=swap" rel="stylesheet" media="print" onload="this.media='all'" />
    <link rel="stylesheet" href="/css/ox" />
</head>
<body>
    <div id="app" data-app="${slug}"></div>
    <script type="module">
        const v = Date.now();
        const { default: init } = await import('/js/${slug}.js?v=' + v);
        await init('/wasm/${slug}_bg.wasm?v=' + v);
    </script>
</body>
</html>`;
await mutate("pages:publish", { slug, html, secret });

console.log("\n  done.");
