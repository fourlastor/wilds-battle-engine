-- Move 54. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "mist", name = "Mist", type = Type.Ice,
  category = Category.Status, pp = 30, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"snatch"},
  script = function(ctx)
    if not ctx:start_effect(ctx.user.team, "mist", {turns = 5, rules = {{kind = Rule.BlockStatDrops}}}) then ctx:fail() end
  end,
}
