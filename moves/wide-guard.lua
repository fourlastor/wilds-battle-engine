-- Move 469. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "wide-guard", name = "Wide Guard", type = Type.Rock,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 3, target = Target.User,
  flags = {"snatch"},
  script = function(ctx)
    if #ctx.pending == 0 or ctx.user.team.effects.wide_guard then ctx:fail(); return end
    ctx:start_effect(ctx.user.team, "wide_guard", {turns=1, rules={{kind=Rule.Guard, spread_only=true}}})
    ctx:record_protection()
  end,
}
