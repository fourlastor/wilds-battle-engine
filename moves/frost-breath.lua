-- Move 524: Frost Breath. Identity and parameters: pokemon_data/assets/moves/524.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 60, always_crit = true}
}}

move.id = "frost-breath"
move.name = "Frost Breath"
move.type = Type.Ice
move.category = Category.Special
move.pp = 10
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
