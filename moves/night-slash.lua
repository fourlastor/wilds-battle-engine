-- Move 400: Night Slash. Identity and parameters: pokemon_data/assets/moves/400.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 70, high_crit = true}
}}

move.id = "night-slash"
move.name = "Night Slash"
move.type = Type.Dark
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "slicing"}
return move
