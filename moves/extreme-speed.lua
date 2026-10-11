-- Move 245: Extreme Speed. Identity and parameters: pokemon_data/assets/moves/245.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80}
}}

move.id = "extreme-speed"
move.name = "Extreme Speed"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 5
move.accuracy = 1
move.priority = 2
move.target = Target.Selected
move.flags = {"contact"}
return move
