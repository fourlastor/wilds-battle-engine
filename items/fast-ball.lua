local function modifier(ctx, target)
    return (target.base_speed or 0) >= 100 and 4 or 1
end

return {
  id = "fast-ball", name = "Fast Ball", targets = "foe",
  usable = function(_, ctx) return ctx.wild_battle end,
  script = function(ctx) ctx:attempt_capture(ctx.target, modifier(ctx, ctx.target)) end,
}
