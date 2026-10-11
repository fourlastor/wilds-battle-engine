-- Move 417: Nasty Plot. Identity and parameters: pokemon_data/assets/moves/417.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.SpAttack] = 2}}
}}

move.id = "nasty-plot"
move.name = "Nasty Plot"
move.type = Type.Dark
move.category = Category.Status
move.pp = 20
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch"}
return move
