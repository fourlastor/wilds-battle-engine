-- Move 438: Power Whip. Identity and parameters: pokemon_data/assets/moves/438.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 120}
}}

move.id = "power-whip"
move.name = "Power Whip"
move.type = Type.Grass
move.category = Category.Physical
move.pp = 10
move.accuracy = 0.85
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
