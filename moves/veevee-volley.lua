-- Move 741. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "veevee-volley", name = "Veevee Volley", type = Type.Normal,
  category = Category.Physical, pp = 20, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {"contact"},
  script = function(ctx)
    ctx:damage(math.max(1, math.floor(((ctx.user.happiness or 255)) * 2 / 5)))
  end,
}
