-- Move 113. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "light-screen", name = "Light Screen", type = Type.Psychic,
  category = Category.Status, pp = 30, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"snatch"},
  script = function(ctx)
    if not ctx:start_effect(ctx.user.team, "light_screen", {turns = 5, rules = {{kind = Rule.DamageTaken, category = Category.Special, factor = 0.5, multi_battle_factor = 2732 / 4096, not_on_crit = true}}}) then ctx:fail() end
  end,
}
