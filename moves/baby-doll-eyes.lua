-- Move 608: Baby-Doll Eyes. Identity and parameters: pokemon_data/assets/moves/608.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Attack] = -1}}
}}

move.id = "baby-doll-eyes"
move.name = "Baby-Doll Eyes"
move.type = Type.Fairy
move.category = Category.Status
move.pp = 30
move.accuracy = 1
move.priority = 1
move.target = Target.Selected
move.flags = {"reflectable"}
return move
