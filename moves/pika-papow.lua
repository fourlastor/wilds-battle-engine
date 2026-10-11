-- Move 732. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "pika-papow", name = "Pika Papow", type = Type.Electric,
  category = Category.Special, pp = 20, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {},
  script = function(ctx)
    ctx:damage(math.max(1, math.floor(((ctx.user.happiness or 255)) * 2 / 5)))
  end,
}
