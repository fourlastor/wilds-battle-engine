-- Move 465: Seed Flare. Identity and parameters: pokemon_data/assets/moves/465.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 120},
  {kind = Effect.Stats, chance = 0.4, self = false, stages = {[Stat.SpDefense] = -2}}
}}

move.id = "seed-flare"
move.name = "Seed Flare"
move.type = Type.Grass
move.category = Category.Special
move.pp = 5
move.accuracy = 0.85
move.priority = 0
move.target = Target.Selected
move.flags = {"sheer_force"}
return move
