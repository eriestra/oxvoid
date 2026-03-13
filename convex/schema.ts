import { defineSchema, defineTable } from "convex/server";
import { v } from "convex/values";

export default defineSchema({
  // Published apps — HTML shell + WASM reference
  pages: defineTable({
    slug: v.string(),
    html: v.string(),
    updatedAt: v.number(),
  }).index("by_slug", ["slug"]),

  // Binary/text assets — WASM, JS glue, CSS
  assets: defineTable({
    name: v.string(),
    content: v.string(), // base64 for binary, raw for text
    contentType: v.string(),
    updatedAt: v.number(),
  }).index("by_name", ["name"]),

  // Publish secret
  keys: defineTable({
    name: v.string(),
    value: v.string(),
  }).index("by_name", ["name"]),
});
