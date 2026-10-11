-- Move 667: High Horsepower. Identity and parameters: pokemon_data/assets/moves/667.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 95}
}}

move.id = "high-horsepower"
move.name = "High Horsepower"
move.type = Type.Ground
move.category = Category.Physical
move.pp = 10
move.accuracy = 0.95
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
