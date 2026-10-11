-- Move 124: Sludge. Identity and parameters: pokemon_data/assets/moves/124.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 65},
  {kind = Effect.Status, status = Status.Poisoned, chance = 0.3}
}}

move.id = "sludge"
move.name = "Sludge"
move.type = Type.Poison
move.category = Category.Special
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"sheer_force"}
return move
