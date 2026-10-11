-- Move 8: Ice Punch. Identity and parameters: pokemon_data/assets/moves/8.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 75},
  {kind = Effect.Status, status = Status.Frozen, chance = 0.1}
}}

move.id = "ice-punch"
move.name = "Ice Punch"
move.type = Type.Ice
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "punch"}
return move
