-- Move 688: Trop Kick. Identity and parameters: pokemon_data/assets/moves/688.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 70},
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Attack] = -1}}
}}

move.id = "trop-kick"
move.name = "Trop Kick"
move.type = Type.Grass
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "sheer_force"}
return move
