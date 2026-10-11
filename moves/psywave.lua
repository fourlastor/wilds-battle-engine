-- Move 149. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "psywave", name = "Psywave", type = Type.Psychic,
  category = Category.Special, pp = 15, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    ctx:direct_damage(ctx.target, math.max(1, math.floor(ctx:random_int(50, 150) * ctx.user.level / 100)))
  end,
}
