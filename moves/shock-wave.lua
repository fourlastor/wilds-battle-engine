-- Move 351: Shock Wave. Identity and parameters: pokemon_data/assets/moves/351.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 60}
}}

move.id = "shock-wave"
move.name = "Shock Wave"
move.type = Type.Electric
move.category = Category.Special
move.pp = 20
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
