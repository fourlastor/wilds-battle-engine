-- Move 25: Mega Kick. Identity and parameters: pokemon_data/assets/moves/25.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 120}
}}

move.id = "mega-kick"
move.name = "Mega Kick"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 5
move.accuracy = 0.75
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
