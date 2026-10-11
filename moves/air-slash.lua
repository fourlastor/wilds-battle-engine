-- Move 403: Air Slash. Identity and parameters: pokemon_data/assets/moves/403.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 75},
  {kind = Effect.Flinch, chance = 0.3}
}}

move.id = "air-slash"
move.name = "Air Slash"
move.type = Type.Flying
move.category = Category.Special
move.pp = 15
move.accuracy = 0.95
move.priority = 0
move.target = Target.Selected
move.flags = {"slicing"}
return move
