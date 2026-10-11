-- Move 821: Freezing Glare. Identity and parameters: pokemon_data/assets/moves/821.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 90},
  {kind = Effect.Status, status = Status.Frozen, chance = 0.1}
}}

move.id = "freezing-glare"
move.name = "Freezing Glare"
move.type = Type.Psychic
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
