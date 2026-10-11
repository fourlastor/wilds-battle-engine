-- Move 70: Strength. Identity and parameters: pokemon_data/assets/moves/70.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80}
}}

move.id = "strength"
move.name = "Strength"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
