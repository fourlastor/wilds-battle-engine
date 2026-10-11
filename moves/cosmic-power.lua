-- Move 322: Cosmic Power. Identity and parameters: pokemon_data/assets/moves/322.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Defense] = 1, [Stat.SpDefense] = 1}}
}}

move.id = "cosmic-power"
move.name = "Cosmic Power"
move.type = Type.Psychic
move.category = Category.Status
move.pp = 20
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch"}
return move
