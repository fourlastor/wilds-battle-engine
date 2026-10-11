-- Move 216. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "return", name = "Return", type = Type.Normal,
  category = Category.Physical, pp = 20, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"contact"},
  script = function(ctx)
    ctx:damage(math.max(1, math.floor(((ctx.user.happiness or 255)) * 2 / 5)))
  end,
}
