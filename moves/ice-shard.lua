-- Move 420: Ice Shard. Identity and parameters: pokemon_data/assets/moves/420.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 40}
}}

move.id = "ice-shard"
move.name = "Ice Shard"
move.type = Type.Ice
move.category = Category.Physical
move.pp = 30
move.accuracy = 1
move.priority = 1
move.target = Target.Selected
move.flags = {}
return move
