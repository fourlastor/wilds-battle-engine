-- Move 69: identity and parameters from pokemon_data/assets/moves/69.json.
return {
  id = "seismic-toss", name = "Seismic Toss",
  type = Type.Fighting, category = Category.Physical,
  pp = 20, accuracy = 1, priority = 0, target = Target.Selected,
  flags = {"contact"},
  effects = {{kind = Effect.LevelDamage}},
}
