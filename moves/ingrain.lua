-- Move 275. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "ingrain", name = "Ingrain", type = Type.Grass,
  category = Category.Status, pp = 20, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"snatch"},
  script = function(ctx)
    if not ctx:start_effect(ctx.user, "ingrain", {rules = {{kind = Rule.HealEachTurn, fraction = 1/16}, {kind = Rule.Trapped}, {kind = Rule.BlockForcedSwitch}, {kind = Rule.Grounded}}}) then ctx:fail() end
  end,
}
