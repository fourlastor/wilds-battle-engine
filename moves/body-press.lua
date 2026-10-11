-- Move 776: identity and parameters from pokemon_data/assets/moves/776.json.
return {
  id = "body-press", name = "Body Press",
  type = Type.Fighting, category = Category.Physical,
  pp = 10, accuracy = 1, priority = 0, target = Target.Selected,
  flags = {"contact"},
  script = function(ctx) ctx:damage(80, {attack_stat = Stat.Defense}) end,
}
