-- Move 715: Tearful Look. Identity and parameters: pokemon_data/assets/moves/715.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Attack] = -1, [Stat.SpAttack] = -1}}
}}

move.id = "tearful-look"
move.name = "Tearful Look"
move.type = Type.Normal
move.category = Category.Status
move.pp = 20
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {"reflectable"}
return move
