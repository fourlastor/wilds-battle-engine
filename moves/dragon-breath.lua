-- Move 225: Dragon Breath. Identity and parameters: pokemon_data/assets/moves/225.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 60},
  {kind = Effect.Status, status = Status.Paralyzed, chance = 0.3}
}}

move.id = "dragon-breath"
move.name = "Dragon Breath"
move.type = Type.Dragon
move.category = Category.Special
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"sheer_force"}
return move
