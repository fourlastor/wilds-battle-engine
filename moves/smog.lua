-- Move 123: Smog. Identity and parameters: pokemon_data/assets/moves/123.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 30},
  {kind = Effect.Status, status = Status.Poisoned, chance = 0.4}
}}

move.id = "smog"
move.name = "Smog"
move.type = Type.Poison
move.category = Category.Special
move.pp = 20
move.accuracy = 0.7
move.priority = 0
move.target = Target.Selected
move.flags = {"sheer_force"}
return move
