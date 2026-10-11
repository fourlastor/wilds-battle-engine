-- Move 115. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "reflect", name = "Reflect", type = Type.Psychic,
  category = Category.Status, pp = 20, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"snatch"},
  script = function(ctx)
    if not ctx:start_effect(ctx.user.team, "reflect", {turns = 5, rules = {{kind = Rule.DamageTaken, category = Category.Physical, factor = 0.5, multi_battle_factor = 2732 / 4096, not_on_crit = true, screen = true}}}) then ctx:fail() end
  end,
}
