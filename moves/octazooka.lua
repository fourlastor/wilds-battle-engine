-- Move 190: Octazooka. Identity and parameters: pokemon_data/assets/moves/190.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 65},
  {kind = Effect.Stats, chance = 0.5, self = false, stages = {[Stat.Accuracy] = -1}}
}}

move.id = "octazooka"
move.name = "Octazooka"
move.type = Type.Water
move.category = Category.Special
move.pp = 10
move.accuracy = 0.85
move.priority = 0
move.target = Target.Selected
move.flags = {"bullet"}
return move
