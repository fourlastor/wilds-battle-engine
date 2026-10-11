-- Move 305: Poison Fang. Identity and parameters: pokemon_data/assets/moves/305.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 50},
  {kind = Effect.Status, status = Status.BadlyPoisoned, chance = 0.5}
}}

move.id = "poison-fang"
move.name = "Poison Fang"
move.type = Type.Poison
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "bite", "sheer_force"}
return move
