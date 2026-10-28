local function modifier(ctx, target)
    return ctx.battle_turn == 1 and 5 or 1
end

return {
  id = "quick-ball", name = "Quick Ball", targets = "foe",
  usable = function(_, ctx) return ctx.wild_battle end,
  script = function(ctx) ctx:attempt_capture(ctx.target, modifier(ctx, ctx.target)) end,
}
