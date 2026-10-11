-- Move 360. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "gyro-ball", name = "Gyro Ball", type = Type.Steel,
  category = Category.Physical, pp = 5, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"contact", "bullet"},
  script = function(ctx)
    ctx:damage(math.min(150, math.floor(25 * ctx.target.speed / math.max(1, ctx.user.speed)) + 1))
  end,
}
