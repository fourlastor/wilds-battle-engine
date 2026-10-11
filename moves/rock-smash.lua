-- Move 249: Rock Smash. Identity and parameters: pokemon_data/assets/moves/249.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 40},
  {kind = Effect.Stats, chance = 0.5, self = false, stages = {[Stat.Defense] = -1}}
}}

move.id = "rock-smash"
move.name = "Rock Smash"
move.type = Type.Fighting
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
