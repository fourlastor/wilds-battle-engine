-- Move 483: Quiver Dance. Identity and parameters: pokemon_data/assets/moves/483.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.SpAttack] = 1, [Stat.SpDefense] = 1, [Stat.Speed] = 1}}
}}

move.id = "quiver-dance"
move.name = "Quiver Dance"
move.type = Type.Bug
move.category = Category.Status
move.pp = 20
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch", "dance"}
return move
