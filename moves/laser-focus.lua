-- Move 673. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "laser-focus", name = "Laser Focus", type = Type.Normal,
  category = Category.Status, pp = 30, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"snatch"},
  script = function(ctx)
    ctx:start_effect(ctx.user, "laser_focus", {turns = 2, restart = true, rules = {{kind = Rule.CriticalStage, stages = 3}}})
  end,
}
