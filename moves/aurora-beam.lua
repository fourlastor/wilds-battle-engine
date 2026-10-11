-- Move 62: Aurora Beam. Identity and parameters: pokemon_data/assets/moves/62.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 65},
  {kind = Effect.Stats, chance = 0.1, self = false, stages = {[Stat.Attack] = -1}}
}}

move.id = "aurora-beam"
move.name = "Aurora Beam"
move.type = Type.Ice
move.category = Category.Special
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
