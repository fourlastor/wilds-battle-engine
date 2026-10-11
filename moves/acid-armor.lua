-- Move 151: Acid Armor. Identity and parameters: pokemon_data/assets/moves/151.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Defense] = 2}}
}}

move.id = "acid-armor"
move.name = "Acid Armor"
move.type = Type.Poison
move.category = Category.Status
move.pp = 20
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch"}
return move
