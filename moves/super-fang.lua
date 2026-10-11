-- Move 162. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "super-fang", name = "Super Fang", type = Type.Normal,
  category = Category.Physical, pp = 10, accuracy = 0.9,
  priority = 0, target = Target.AnyOther,
  flags = {"contact"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    ctx:direct_damage(ctx.target, math.max(1, math.floor(ctx.target.hp / 2)))
  end,
}
