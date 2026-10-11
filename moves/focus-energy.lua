-- Move 116. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "focus-energy", name = "Focus Energy", type = Type.Normal,
  category = Category.Status, pp = 30, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"snatch"},
  script = function(ctx)
    if not ctx:start_effect(ctx.user, "focus_energy", {rules = {{kind = Rule.CriticalStage, stages = 2}}}) then ctx:fail() end
  end,
}
