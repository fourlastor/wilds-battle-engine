local function modifier(ctx, target)
    -- The host flags species that evolve with a Moon Stone.
    return target.flags.moon_stone and 4 or 1
end

return {
  id = "moon-ball", name = "Moon Ball", targets = "foe",
  usable = function(_, ctx) return ctx.wild_battle end,
  script = function(ctx) ctx:attempt_capture(ctx.target, modifier(ctx, ctx.target)) end,
}
