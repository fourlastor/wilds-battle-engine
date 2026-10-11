-- Move 406: Dragon Pulse. Identity and parameters: pokemon_data/assets/moves/406.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 85}
}}

move.id = "dragon-pulse"
move.name = "Dragon Pulse"
move.type = Type.Dragon
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"pulse"}
return move
