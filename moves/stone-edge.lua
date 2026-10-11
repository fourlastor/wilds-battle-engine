-- Move 444: Stone Edge. Identity and parameters: pokemon_data/assets/moves/444.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 100, high_crit = true}
}}

move.id = "stone-edge"
move.name = "Stone Edge"
move.type = Type.Rock
move.category = Category.Physical
move.pp = 5
move.accuracy = 0.8
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
