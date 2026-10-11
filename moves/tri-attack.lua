-- Move 161. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "tri-attack", name = "Tri Attack", type = Type.Normal,
  category = Category.Special, pp = 10, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {},
  script = function(ctx)
    if ctx:damage(80).hit then
      ctx:apply_status(ctx.target, ({Status.Burned, Status.Paralyzed, Status.Frozen})[ctx:random_int(1, 3)], {chance = 0.2})
    end
  end,
}
