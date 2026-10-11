-- Move 163: Slash. Identity and parameters: pokemon_data/assets/moves/163.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 70, high_crit = true}
}}

move.id = "slash"
move.name = "Slash"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "slicing"}
return move
