-- Move 359: Hammer Arm. Identity and parameters: pokemon_data/assets/moves/359.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 100},
  {kind = Effect.Stats, chance = 1, self = true, stages = {[Stat.Speed] = -1}}
}}

move.id = "hammer-arm"
move.name = "Hammer Arm"
move.type = Type.Fighting
move.category = Category.Physical
move.pp = 10
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "punch"}
return move
