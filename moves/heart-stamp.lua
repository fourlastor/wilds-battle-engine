-- Move 531: Heart Stamp. Identity and parameters: pokemon_data/assets/moves/531.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 60},
  {kind = Effect.Flinch, chance = 0.3}
}}

move.id = "heart-stamp"
move.name = "Heart Stamp"
move.type = Type.Psychic
move.category = Category.Physical
move.pp = 25
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "sheer_force"}
return move
