local function modifier(ctx, target)
    -- The host flags Ultra Beasts.
    return target.flags.ultra_beast and 5 or 410 / 4096
end

return {
  id = "beast-ball", name = "Beast Ball", targets = "foe",
  usable = function(_, ctx) return ctx.wild_battle end,
  script = function(ctx) ctx:attempt_capture(ctx.target, modifier(ctx, ctx.target)) end,
}
