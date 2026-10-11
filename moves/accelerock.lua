-- Move 709: Accelerock. Identity and parameters: pokemon_data/assets/moves/709.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 40}
}}

move.id = "accelerock"
move.name = "Accelerock"
move.type = Type.Rock
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 1
move.target = Target.Selected
move.flags = {"contact"}
return move
