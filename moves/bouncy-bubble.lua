-- Move 733: Bouncy Bubble. Identity and parameters: pokemon_data/assets/moves/733.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 60, drain = 1}
}}

move.id = "bouncy-bubble"
move.name = "Bouncy Bubble"
move.type = Type.Water
move.category = Category.Special
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
