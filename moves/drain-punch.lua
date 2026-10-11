-- Move 409: Drain Punch. Identity and parameters: pokemon_data/assets/moves/409.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 75, drain = 0.5}
}}

move.id = "drain-punch"
move.name = "Drain Punch"
move.type = Type.Fighting
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "punch"}
return move
