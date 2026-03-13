import { mutation } from "./_generated/server";
import { v } from "convex/values";

export const seedSecret = mutation({
  args: { secret: v.string() },
  handler: async (ctx, { secret }) => {
    const existing = await ctx.db
      .query("keys")
      .withIndex("by_name", (q) => q.eq("name", "PUBLISH_SECRET"))
      .unique();
    if (existing) {
      await ctx.db.patch(existing._id, { value: secret });
    } else {
      await ctx.db.insert("keys", { name: "PUBLISH_SECRET", value: secret });
    }
  },
});
