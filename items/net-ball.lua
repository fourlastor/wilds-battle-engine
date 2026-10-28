local function modifier(ctx, target)
    local water_or_bug = false
    for _, kind in ipairs(target.types) do
      if kind == Type.Water or kind == Type.Bug then water_or_bug = true end
    end
    return water_or_bug and 3.5 or 1
end

return {
  id = "net-ball", name = "Net Ball", targets = "foe",
  usable = function(_, ctx) return ctx.wild_battle end,
  script = function(ctx) ctx:attempt_capture(ctx.target, modifier(ctx, ctx.target)) end,
}
