-- Move 179. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "reversal", name = "Reversal", type = Type.Fighting,
  category = Category.Physical, pp = 15, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"contact"},
  script = function(ctx)
    local ratio = math.max(1, math.floor(ctx.user.hp * 48 / ctx.user.max_hp))
    ctx:damage(ratio < 2 and 200 or ratio < 5 and 150 or ratio < 10 and 100 or ratio < 17 and 80 or ratio < 33 and 40 or 20)
  end,
}
