-- Move 489: Coil. Identity and parameters: pokemon_data/assets/moves/489.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Attack] = 1, [Stat.Defense] = 1, [Stat.Accuracy] = 1}}
}}

move.id = "coil"
move.name = "Coil"
move.type = Type.Poison
move.category = Category.Status
move.pp = 20
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch"}
return move
