-- Move 578. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "crafty-shield", name = "Crafty Shield", type = Type.Fairy,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 3, target = Target.User,
  flags = {},
  script = function(ctx)
    if #ctx.pending == 0 or ctx.user.team.effects.crafty_shield then ctx:fail(); return end
    ctx:start_effect(ctx.user.team, "crafty_shield", {turns=1, rules={{kind=Rule.Guard, status_only=true, unbypassable=true}}})
  end,
}
