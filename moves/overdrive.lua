-- Move 786: identity and parameters from pokemon_data/assets/moves/786.json.
return {
  id = "overdrive", name = "Overdrive",
  type = Type.Electric, category = Category.Special,
  pp = 10, accuracy = 1, priority = 0, target = Target.AllOpponents,
  flags = {"sound", "bypass_substitute"},
  script = function(ctx) ctx:damage(80) end,
}
