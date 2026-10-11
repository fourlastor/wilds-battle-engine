-- Move 380. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "gastro-acid", name = "Gastro Acid", type = Type.Poison,
  category = Category.Status, pp = 10, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"reflectable"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    if not ctx:suppress_ability(ctx.target) then ctx:fail() end
  end,
}
