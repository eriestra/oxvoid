import { mutation, query } from "./_generated/server";
import { v } from "convex/values";

async function checkSecret(ctx: any, secret: string) {
  const key = await ctx.db
    .query("keys")
    .withIndex("by_name", (q: any) => q.eq("name", "PUBLISH_SECRET"))
    .unique();
  if (!key || key.value !== secret) throw new Error("Unauthorized");
}

export const set = mutation({
  args: {
    name: v.string(),
    content: v.string(),
    contentType: v.string(),
    secret: v.string(),
  },
  handler: async (ctx, { name, content, contentType, secret }) => {
    await checkSecret(ctx, secret);
    const existing = await ctx.db
      .query("assets")
      .withIndex("by_name", (q) => q.eq("name", name))
      .unique();
    if (existing) {
      await ctx.db.patch(existing._id, { content, contentType, updatedAt: Date.now() });
    } else {
      await ctx.db.insert("assets", { name, content, contentType, updatedAt: Date.now() });
    }
  },
});

export const get = query({
  args: { name: v.string() },
  handler: async (ctx, { name }) => {
    return ctx.db
      .query("assets")
      .withIndex("by_name", (q) => q.eq("name", name))
      .unique();
  },
});
