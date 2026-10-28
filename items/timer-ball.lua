local function modifier(ctx, target)
    return math.min(4, 1 + (ctx.battle_turn - 1) * 1229 / 4096)
end

return {
  id = "timer-ball", name = "Timer Ball", targets = "foe",
  usable = function(_, ctx) return ctx.wild_battle end,
  script = function(ctx) ctx:attempt_capture(ctx.target, modifier(ctx, ctx.target)) end,
}
