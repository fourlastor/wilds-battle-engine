-- Move 583: Play Rough. Identity and parameters: pokemon_data/assets/moves/583.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 90},
  {kind = Effect.Stats, chance = 0.1, self = false, stages = {[Stat.Attack] = -1}}
}}

move.id = "play-rough"
move.name = "Play Rough"
move.type = Type.Fairy
move.category = Category.Physical
move.pp = 10
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "sheer_force"}
return move
