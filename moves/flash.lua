-- Move 148: Flash. Identity and parameters: pokemon_data/assets/moves/148.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Accuracy] = -1}}
}}

move.id = "flash"
move.name = "Flash"
move.type = Type.Normal
move.category = Category.Status
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"reflectable"}
return move
