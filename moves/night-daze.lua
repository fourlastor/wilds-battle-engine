-- Move 539: Night Daze. Identity and parameters: pokemon_data/assets/moves/539.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 85},
  {kind = Effect.Stats, chance = 0.4, self = false, stages = {[Stat.Accuracy] = -1}}
}}

move.id = "night-daze"
move.name = "Night Daze"
move.type = Type.Dark
move.category = Category.Special
move.pp = 10
move.accuracy = 0.95
move.priority = 0
move.target = Target.Selected
move.flags = {"sheer_force"}
return move
