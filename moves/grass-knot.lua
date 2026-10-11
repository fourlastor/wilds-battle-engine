-- Move 447. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "grass-knot", name = "Grass Knot", type = Type.Grass,
  category = Category.Special, pp = 20, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"contact"},
  script = function(ctx)
    local w = ctx.target.weight
    ctx:damage(w >= 200 and 120 or w >= 100 and 100 or w >= 50 and 80 or w >= 25 and 60 or w >= 10 and 40 or 20)
  end,
}
