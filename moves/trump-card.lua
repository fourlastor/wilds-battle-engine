-- Move 376. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "trump-card", name = "Trump Card", type = Type.Normal,
  category = Category.Special, pp = 5, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {"contact"},
  script = function(ctx)
    local pp = ctx.user.pp["trump-card"]
    local power = pp == 0 and 200 or pp == 1 and 80 or pp == 2 and 60 or pp == 3 and 50 or 40
    ctx:damage(power)
  end,
}
