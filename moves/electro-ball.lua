-- Move 486. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "electro-ball", name = "Electro Ball", type = Type.Electric,
  category = Category.Special, pp = 10, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"bullet"},
  script = function(ctx)
    local ratio = math.min(4, math.floor(ctx.user.speed / math.max(1, ctx.target.speed)))
    ctx:damage(({40, 60, 80, 120, 150})[ratio + 1])
  end,
}
