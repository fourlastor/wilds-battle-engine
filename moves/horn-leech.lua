-- Move 532: Horn Leech. Identity and parameters: pokemon_data/assets/moves/532.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 75, drain = 0.5}
}}

move.id = "horn-leech"
move.name = "Horn Leech"
move.type = Type.Grass
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
