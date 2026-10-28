local function modifier(ctx, target)
    return (ctx.context.time_of_day == "night" or ctx.context.cave or ctx.environment == "cave") and 3 or 1
end

return {
  id = "dusk-ball", name = "Dusk Ball", targets = "foe",
  usable = function(_, ctx) return ctx.wild_battle end,
  script = function(ctx) ctx:attempt_capture(ctx.target, modifier(ctx, ctx.target)) end,
}
