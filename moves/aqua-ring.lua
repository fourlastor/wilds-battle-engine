-- Move 392. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "aqua-ring", name = "Aqua Ring", type = Type.Water,
  category = Category.Status, pp = 20, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"snatch"},
  script = function(ctx)
    if not ctx:start_effect(ctx.user, "aqua_ring", {rules = {{kind = Rule.HealEachTurn, fraction = 1/16}}}) then ctx:fail() end
  end,
}
