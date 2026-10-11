-- Move 596. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "spiky-shield", name = "Spiky Shield", type = Type.Grass,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 4, target = Target.User,
  flags = {},
  script = function(ctx)
    if #ctx.pending == 0 or not ctx:check_protection() then ctx:fail(); return end
    ctx:start_effect(ctx.user, "spiky_shield", {turns=1, rules={{kind=Rule.Guard, contact_damage=1/8}}})
  end,
}
