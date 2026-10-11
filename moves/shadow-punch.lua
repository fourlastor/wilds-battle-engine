-- Move 325: Shadow Punch. Identity and parameters: pokemon_data/assets/moves/325.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 60}
}}

move.id = "shadow-punch"
move.name = "Shadow Punch"
move.type = Type.Ghost
move.category = Category.Physical
move.pp = 20
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "punch"}
return move
