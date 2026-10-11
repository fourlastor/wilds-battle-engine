-- Move 321: Tickle. Identity and parameters: pokemon_data/assets/moves/321.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Attack] = -1, [Stat.Defense] = -1}}
}}

move.id = "tickle"
move.name = "Tickle"
move.type = Type.Normal
move.category = Category.Status
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"reflectable"}
return move
