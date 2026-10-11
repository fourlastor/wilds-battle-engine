-- Move 85: Thunderbolt. Identity and parameters: pokemon_data/assets/moves/85.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 90},
  {kind = Effect.Status, status = Status.Paralyzed, chance = 0.1}
}}

move.id = "thunderbolt"
move.name = "Thunderbolt"
move.type = Type.Electric
move.category = Category.Special
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
