-- Move 671. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "spotlight", name = "Spotlight", type = Type.Normal,
  category = Category.Status, pp = 15, accuracy = Accuracy.Always,
  priority = 3, target = Target.AnyOther,
  flags = {"reflectable"},
  script = function(ctx)
    if ctx.active_per_side == 1 then ctx:fail(); return end
    if not ctx:try_hit(ctx.target) then return end
    ctx:start_effect(ctx.target.team, "spotlight_" .. ctx.target.index, {turns = 1, restart = true,
      rules = {{kind = Rule.Attention, to = ctx.target, priority = 2, allow_carried = true}}})
  end,
}
