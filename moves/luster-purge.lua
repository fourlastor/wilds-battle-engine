-- Move 295: Luster Purge. Identity and parameters: pokemon_data/assets/moves/295.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 95},
  {kind = Effect.Stats, chance = 0.5, self = false, stages = {[Stat.SpDefense] = -1}}
}}

move.id = "luster-purge"
move.name = "Luster Purge"
move.type = Type.Psychic
move.category = Category.Special
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
