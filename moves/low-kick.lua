-- Move 67. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "low-kick", name = "Low Kick", type = Type.Fighting,
  category = Category.Physical, pp = 20, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"contact"},
  script = function(ctx)
    local w = ctx.target.weight
    local power = w >= 200 and 120 or w >= 100 and 100 or w >= 50 and 80 or w >= 25 and 60 or w >= 10 and 40 or 20
    ctx:damage(power)
  end,
}
