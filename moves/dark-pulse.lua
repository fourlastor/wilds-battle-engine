-- Move 399: Dark Pulse. Identity and parameters: pokemon_data/assets/moves/399.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80},
  {kind = Effect.Flinch, chance = 0.2}
}}

move.id = "dark-pulse"
move.name = "Dark Pulse"
move.type = Type.Dark
move.category = Category.Special
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"pulse", "sheer_force"}
return move
