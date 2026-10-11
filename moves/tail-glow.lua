-- Move 294: Tail Glow. Identity and parameters: pokemon_data/assets/moves/294.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.SpAttack] = 3}}
}}

move.id = "tail-glow"
move.name = "Tail Glow"
move.type = Type.Bug
move.category = Category.Status
move.pp = 20
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch"}
return move
