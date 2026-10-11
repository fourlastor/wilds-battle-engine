-- Move 393. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "magnet-rise", name = "Magnet Rise", type = Type.Electric,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"snatch", "gravity"},
  script = function(ctx)
    if ctx.user.grounded then ctx:fail(); return end
    if not ctx:start_effect(ctx.user, "magnet_rise", {turns = 5, rules = {{kind = Rule.Airborne}}}) then ctx:fail() end
  end,
}
