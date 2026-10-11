-- Move 606. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "celebrate", name = "Celebrate", type = Type.Normal,
  category = Category.Status, pp = 40, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {},
  script = function(ctx)
    ctx:message("Congratulations!")
  end,
}
