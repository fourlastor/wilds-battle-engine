-- Move 587. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "fairy-lock", name = "Fairy Lock", type = Type.Fairy,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.Field,
  flags = {"bypass_substitute", "pressure_all"},
  script = function(ctx)
    if not ctx:start_effect(ctx.field, "fairy_lock", {turns = 2, rules = {{kind = Rule.Trapped}}}) then ctx:fail() end
  end,
}
