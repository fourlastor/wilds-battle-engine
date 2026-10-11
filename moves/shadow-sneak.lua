-- Move 425: Shadow Sneak. Identity and parameters: pokemon_data/assets/moves/425.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 40}
}}

move.id = "shadow-sneak"
move.name = "Shadow Sneak"
move.type = Type.Ghost
move.category = Category.Physical
move.pp = 30
move.accuracy = 1
move.priority = 1
move.target = Target.Selected
move.flags = {"contact"}
return move
