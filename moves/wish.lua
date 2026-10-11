-- Move 273. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "wish", name = "Wish", type = Type.Normal,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"snatch", "heal"},
  script = function(ctx)
    if not ctx:schedule_heal(ctx.user, math.max(1, math.floor(ctx.user.max_hp/2)), 1) then ctx:fail() end
  end,
}
