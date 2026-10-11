-- Move 326: Extrasensory. Identity and parameters: pokemon_data/assets/moves/326.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80},
  {kind = Effect.Flinch, chance = 0.1}
}}

move.id = "extrasensory"
move.name = "Extrasensory"
move.type = Type.Psychic
move.category = Category.Special
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
