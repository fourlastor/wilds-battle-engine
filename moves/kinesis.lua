-- Move 134: Kinesis. Identity and parameters: pokemon_data/assets/moves/134.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Accuracy] = -1}}
}}

move.id = "kinesis"
move.name = "Kinesis"
move.type = Type.Psychic
move.category = Category.Status
move.pp = 15
move.accuracy = 0.8
move.priority = 0
move.target = Target.Selected
move.flags = {"reflectable"}
return move
