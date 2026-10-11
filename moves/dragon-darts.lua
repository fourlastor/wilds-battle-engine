-- Move 751: identity and parameters from pokemon_data/assets/moves/751.json.
return {
  base_power = 50,
  id = "dragon-darts", name = "Dragon Darts",
  type = Type.Dragon, category = Category.Physical,
  pp = 10, accuracy = 1, priority = 0, target = Target.Selected,
  flags = {"smart_target"},
  script = function(ctx) ctx:multi_hit(50, 2, 2) end,
}
