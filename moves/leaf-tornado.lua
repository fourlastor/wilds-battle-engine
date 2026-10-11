-- Move 536: Leaf Tornado. Identity and parameters: pokemon_data/assets/moves/536.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 65},
  {kind = Effect.Stats, chance = 0.5, self = false, stages = {[Stat.Accuracy] = -1}}
}}

move.id = "leaf-tornado"
move.name = "Leaf Tornado"
move.type = Type.Grass
move.category = Category.Special
move.pp = 10
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
