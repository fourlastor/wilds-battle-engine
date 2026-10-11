-- Move 454: Attack Order. Identity and parameters: pokemon_data/assets/moves/454.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 90, high_crit = true}
}}

move.id = "attack-order"
move.name = "Attack Order"
move.type = Type.Bug
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
