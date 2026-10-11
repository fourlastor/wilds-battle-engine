-- Move 141: Leech Life. Identity and parameters: pokemon_data/assets/moves/141.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80, drain = 0.5}
}}

move.id = "leech-life"
move.name = "Leech Life"
move.type = Type.Bug
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "heal"}
return move
