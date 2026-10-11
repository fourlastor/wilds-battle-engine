-- Move 694. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "aurora-veil", name = "Aurora Veil", type = Type.Ice,
  category = Category.Status, pp = 20, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"snatch"},
  script = function(ctx)
    if ctx.weather ~= Weather.Hail then ctx:fail(); return end
    if not ctx:start_effect(ctx.user.team, "aurora_veil", {turns = 5, rules = {
      {kind = Rule.DamageTaken, category = Category.Physical, factor = 0.5, multi_battle_factor = 2732/4096, not_on_crit = true, screen = true, unless_side_effect = "reflect"},
      {kind = Rule.DamageTaken, category = Category.Special, factor = 0.5, multi_battle_factor = 2732/4096, not_on_crit = true, screen = true, unless_side_effect = "light_screen"}
    }}) then ctx:fail() end
  end,
}
