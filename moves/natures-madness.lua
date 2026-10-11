-- Move 717. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "natures-madness", name = "Nature’s Madness", type = Type.Fairy,
  category = Category.Special, pp = 10, accuracy = 0.9,
  priority = 0, target = Target.AnyOther,
  flags = {},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    ctx:direct_damage(ctx.target, math.max(1, math.floor(ctx.target.hp / 2)))
  end,
}
