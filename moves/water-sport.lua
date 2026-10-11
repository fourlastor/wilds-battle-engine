-- Move 346. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "water-sport", name = "Water Sport", type = Type.Water,
  category = Category.Status, pp = 15, accuracy = Accuracy.Always,
  priority = 0, target = Target.Field,
  flags = {"pressure_all"},
  script = function(ctx)
    if not ctx:start_effect(ctx.field, "water_sport", {turns = 5, rules = {
      {kind = Rule.BasePower, type = Type.Fire, factor = 1352/4096}
    }}) then ctx:fail() end
  end,
}
