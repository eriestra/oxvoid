import { mutation, query } from "./_generated/server";
import { v } from "convex/values";

async function checkSecret(ctx: any, secret: string) {
  const key = await ctx.db
    .query("keys")
    .withIndex("by_name", (q: any) => q.eq("name", "PUBLISH_SECRET"))
    .unique();
  if (!key || key.value !== secret) throw new Error("Unauthorized");
}

export const publish = mutation({
  args: { slug: v.string(), html: v.string(), secret: v.string() },
  handler: async (ctx, { slug, html, secret }) => {
    await checkSecret(ctx, secret);
    const existing = await ctx.db
      .query("pages")
      .withIndex("by_slug", (q) => q.eq("slug", slug))
      .unique();
    if (existing) {
      await ctx.db.patch(existing._id, { html, updatedAt: Date.now() });
    } else {
      await ctx.db.insert("pages", { slug, html, updatedAt: Date.now() });
    }
  },
});

export const get = query({
  args: { slug: v.string() },
  handler: async (ctx, { slug }) => {
    return ctx.db
      .query("pages")
      .withIndex("by_slug", (q) => q.eq("slug", slug))
      .unique();
  },
});

export const remove = mutation({
  args: { slug: v.string(), secret: v.string() },
  handler: async (ctx, { slug, secret }) => {
    await checkSecret(ctx, secret);
    const page = await ctx.db
      .query("pages")
      .withIndex("by_slug", (q) => q.eq("slug", slug))
      .unique();
    if (page) await ctx.db.delete(page._id);
  },
});
