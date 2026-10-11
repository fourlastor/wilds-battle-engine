-- Move 197. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "detect", name = "Detect", type = Type.Fighting,
  category = Category.Status, pp = 5, accuracy = Accuracy.Always,
  priority = 4, target = Target.User,
  flags = {},
  script = function(ctx)
    if #ctx.pending == 0 then ctx:fail(); return end
    ctx:protect()
  end,
}
