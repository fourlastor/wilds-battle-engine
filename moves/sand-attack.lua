-- Move 28: Sand Attack. Identity and parameters: pokemon_data/assets/moves/28.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Accuracy] = -1}}
}}

move.id = "sand-attack"
move.name = "Sand Attack"
move.type = Type.Ground
move.category = Category.Status
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"reflectable"}
return move
