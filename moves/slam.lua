-- Move 21: Slam. Identity and parameters: pokemon_data/assets/moves/21.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80}
}}

move.id = "slam"
move.name = "Slam"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 20
move.accuracy = 0.75
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
