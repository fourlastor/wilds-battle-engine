-- Move 10: Scratch. Identity and parameters: pokemon_data/assets/moves/10.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 40}
}}

move.id = "scratch"
move.name = "Scratch"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 35
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
