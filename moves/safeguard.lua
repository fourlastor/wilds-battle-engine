-- Move 219. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "safeguard", name = "Safeguard", type = Type.Normal,
  category = Category.Status, pp = 25, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"snatch"},
  script = function(ctx)
    if not ctx:start_effect(ctx.user.team, "safeguard", {turns = 5, rules = {{kind = Rule.BlockStatus, confusion = true}}}) then ctx:fail() end
  end,
}
