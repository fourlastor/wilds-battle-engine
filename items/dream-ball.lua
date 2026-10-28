local function modifier(ctx, target)
    return target.status == Status.Asleep and 4 or 1
end

return {
  id = "dream-ball", name = "Dream Ball", targets = "foe",
  usable = function(_, ctx) return ctx.wild_battle end,
  script = function(ctx) ctx:attempt_capture(ctx.target, modifier(ctx, ctx.target)) end,
}
