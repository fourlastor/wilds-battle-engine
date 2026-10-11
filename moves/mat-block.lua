-- Move 561. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "mat-block", name = "Mat Block", type = Type.Fighting,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"snatch"},
  script = function(ctx)
    if ctx.user.turns_on_field > 1 or #ctx.pending == 0 or ctx.user.team.effects.mat_block then ctx:fail(); return end
    ctx:start_effect(ctx.user.team, "mat_block", {turns=1, rules={{kind=Rule.Guard, damaging_only=true}}})
  end,
}
