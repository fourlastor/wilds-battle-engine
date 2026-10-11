-- Move 781: identity and parameters from pokemon_data/assets/moves/781.json.
-- The Dynamax-only damage bonus is outside the project scope.
return {
  id = "behemoth-blade", name = "Behemoth Blade",
  type = Type.Steel, category = Category.Physical,
  pp = 5, accuracy = 1, priority = 0, target = Target.Selected,
  flags = {"contact", "slicing"},
  effects = {{kind = Effect.Damage, power = 100}},
}
