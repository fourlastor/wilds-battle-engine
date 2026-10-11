-- Move 297: Feather Dance. Identity and parameters: pokemon_data/assets/moves/297.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Attack] = -2}}
}}

move.id = "feather-dance"
move.name = "Feather Dance"
move.type = Type.Flying
move.category = Category.Status
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"reflectable", "dance"}
return move
