-- Move 49: identity and parameters from pokemon_data/assets/moves/49.json.
return {
  id = "sonic-boom", name = "Sonic Boom",
  type = Type.Normal, category = Category.Special,
  pp = 20, accuracy = 0.9, priority = 0, target = Target.Selected,
  effects = {{kind = Effect.FixedDamage, amount = 20}},
}
