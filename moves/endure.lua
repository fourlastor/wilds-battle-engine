-- Move 203. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "endure", name = "Endure", type = Type.Normal,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 4, target = Target.User,
  flags = {},
  script = function(ctx)
    if #ctx.pending == 0 or not ctx:check_protection() then ctx:fail(); return end
    ctx:start_effect(ctx.user, "endure", {turns = 1, rules = {{kind = Rule.Endure}}})
  end,
}
