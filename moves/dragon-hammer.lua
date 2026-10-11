-- Move 692: Dragon Hammer. Identity and parameters: pokemon_data/assets/moves/692.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 90}
}}

move.id = "dragon-hammer"
move.name = "Dragon Hammer"
move.type = Type.Dragon
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
