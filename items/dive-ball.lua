local function modifier(ctx, target)
    local here = ctx.context
    return (here.surfing or here.fishing or here.underwater) and 3.5 or 1
end

return {
  id = "dive-ball", name = "Dive Ball", targets = "foe",
  usable = function(_, ctx) return ctx.wild_battle end,
  script = function(ctx) ctx:attempt_capture(ctx.target, modifier(ctx, ctx.target)) end,
}
