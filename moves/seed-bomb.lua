-- Move 402: Seed Bomb. Identity and parameters: pokemon_data/assets/moves/402.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80}
}}

move.id = "seed-bomb"
move.name = "Seed Bomb"
move.type = Type.Grass
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"bullet"}
return move
