-- Move 184: Scary Face. Identity and parameters: pokemon_data/assets/moves/184.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Speed] = -2}}
}}

move.id = "scary-face"
move.name = "Scary Face"
move.type = Type.Normal
move.category = Category.Status
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"reflectable"}
return move
