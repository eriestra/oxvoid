import { httpRouter } from "convex/server";
import { httpAction } from "./_generated/server";
import { api } from "./_generated/api";

const http = httpRouter();

// Serve app by slug: GET /app/:slug
http.route({
  pathPrefix: "/app/",
  method: "GET",
  handler: httpAction(async (ctx, request) => {
    const url = new URL(request.url);
    const slug = url.pathname.split("/app/")[1];
    if (!slug) return new Response("Not found", { status: 404 });

    const page = await ctx.runQuery(api.pages.get, { slug });
    if (!page) return new Response(`App "${slug}" not found`, { status: 404 });

    return new Response(page.html, {
      headers: {
        "Content-Type": "text/html; charset=utf-8",
        "Cache-Control": "no-cache",
      },
    });
  }),
});

// Serve CSS: GET /css/:name
http.route({
  pathPrefix: "/css/",
  method: "GET",
  handler: httpAction(async (ctx, request) => {
    const url = new URL(request.url);
    const name = url.pathname.split("/css/")[1];
    const asset = await ctx.runQuery(api.assets.get, { name: `css/${name}` });
    if (!asset) return new Response("Not found", { status: 404 });

    return new Response(asset.content, {
      headers: {
        "Content-Type": "text/css; charset=utf-8",
        "Cache-Control": "no-cache",
      },
    });
  }),
});

// Serve JS glue: GET /js/:name
http.route({
  pathPrefix: "/js/",
  method: "GET",
  handler: httpAction(async (ctx, request) => {
    const url = new URL(request.url);
    const name = url.pathname.split("/js/")[1];
    const asset = await ctx.runQuery(api.assets.get, { name: `js/${name}` });
    if (!asset) return new Response("Not found", { status: 404 });

    return new Response(asset.content, {
      headers: {
        "Content-Type": "application/javascript; charset=utf-8",
        "Cache-Control": "no-cache",
      },
    });
  }),
});

// Serve WASM: GET /wasm/:name
http.route({
  pathPrefix: "/wasm/",
  method: "GET",
  handler: httpAction(async (ctx, request) => {
    const url = new URL(request.url);
    const name = url.pathname.split("/wasm/")[1];
    const asset = await ctx.runQuery(api.assets.get, { name: `wasm/${name}` });
    if (!asset) return new Response("Not found", { status: 404 });

    // Asset is base64-encoded binary
    const binary = Uint8Array.from(atob(asset.content), (c) => c.charCodeAt(0));
    return new Response(binary, {
      headers: {
        "Content-Type": "application/wasm",
        "Cache-Control": "no-cache",
      },
    });
  }),
});

export default http;
