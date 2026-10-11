-- Move 337: Dragon Claw. Identity and parameters: pokemon_data/assets/moves/337.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80}
}}

move.id = "dragon-claw"
move.name = "Dragon Claw"
move.type = Type.Dragon
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
