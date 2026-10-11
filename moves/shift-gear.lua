-- Move 508: Shift Gear. Identity and parameters: pokemon_data/assets/moves/508.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Attack] = 1, [Stat.Speed] = 2}}
}}

move.id = "shift-gear"
move.name = "Shift Gear"
move.type = Type.Steel
move.category = Category.Status
move.pp = 10
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch"}
return move
