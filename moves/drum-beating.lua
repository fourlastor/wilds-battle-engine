-- Move 778: Drum Beating. Identity and parameters: pokemon_data/assets/moves/778.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80},
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Speed] = -1}}
}}

move.id = "drum-beating"
move.name = "Drum Beating"
move.type = Type.Grass
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"sheer_force"}
return move
