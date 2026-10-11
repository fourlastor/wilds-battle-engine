-- Move 782: identity and parameters from pokemon_data/assets/moves/782.json.
-- The Dynamax-only damage bonus is outside the project scope.
return {
  id = "behemoth-bash", name = "Behemoth Bash",
  type = Type.Steel, category = Category.Physical,
  pp = 5, accuracy = 1, priority = 0, target = Target.Selected,
  flags = {"contact"},
  effects = {{kind = Effect.Damage, power = 100}},
}
