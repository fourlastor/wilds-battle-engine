-- Move 501. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "quick-guard", name = "Quick Guard", type = Type.Fighting,
  category = Category.Status, pp = 15, accuracy = Accuracy.Always,
  priority = 3, target = Target.User,
  flags = {"snatch"},
  script = function(ctx)
    if #ctx.pending == 0 or ctx.user.team.effects.quick_guard then ctx:fail(); return end
    ctx:start_effect(ctx.user.team, "quick_guard", {turns=1, rules={{kind=Rule.Guard, priority_only=true}}})
    ctx:record_protection()
  end,
}
