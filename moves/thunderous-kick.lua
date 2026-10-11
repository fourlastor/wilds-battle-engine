-- Move 823: Thunderous Kick. Identity and parameters: pokemon_data/assets/moves/823.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 90},
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Defense] = -1}}
}}

move.id = "thunderous-kick"
move.name = "Thunderous Kick"
move.type = Type.Fighting
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "sheer_force"}
return move
