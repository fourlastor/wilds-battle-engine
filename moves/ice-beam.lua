-- Move 58: Ice Beam. Identity and parameters: pokemon_data/assets/moves/58.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 90},
  {kind = Effect.Status, status = Status.Frozen, chance = 0.1}
}}

move.id = "ice-beam"
move.name = "Ice Beam"
move.type = Type.Ice
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
