local function modifier(ctx, target)
    return target.level < 30 and (41 - target.level) / 10 or 1
end

return {
  id = "nest-ball", name = "Nest Ball", targets = "foe",
  usable = function(_, ctx) return ctx.wild_battle end,
  script = function(ctx) ctx:attempt_capture(ctx.target, modifier(ctx, ctx.target)) end,
}
