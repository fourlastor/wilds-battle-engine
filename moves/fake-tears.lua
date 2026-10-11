-- Move 313: Fake Tears. Identity and parameters: pokemon_data/assets/moves/313.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.SpDefense] = -2}}
}}

move.id = "fake-tears"
move.name = "Fake Tears"
move.type = Type.Dark
move.category = Category.Status
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"reflectable"}
return move
