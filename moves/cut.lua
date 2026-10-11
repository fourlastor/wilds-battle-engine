-- Move 15: Cut. Identity and parameters: pokemon_data/assets/moves/15.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 50}
}}

move.id = "cut"
move.name = "Cut"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 30
move.accuracy = 0.95
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "slicing"}
return move
