local function modifier(ctx, target)
    -- The host flags species the player has caught before.
    return target.flags.registered and 3.5 or 1
end

return {
  id = "repeat-ball", name = "Repeat Ball", targets = "foe",
  usable = function(_, ctx) return ctx.wild_battle end,
  script = function(ctx) ctx:attempt_capture(ctx.target, modifier(ctx, ctx.target)) end,
}
