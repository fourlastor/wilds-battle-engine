-- Move 300. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "mud-sport", name = "Mud Sport", type = Type.Ground,
  category = Category.Status, pp = 15, accuracy = Accuracy.Always,
  priority = 0, target = Target.Field,
  flags = {},
  script = function(ctx)
    if not ctx:start_effect(ctx.field, "mud_sport", {turns = 5, rules = {
      {kind = Rule.BasePower, type = Type.Electric, factor = 1352/4096}
    }}) then ctx:fail() end
  end,
}
