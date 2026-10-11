-- Move 192: Zap Cannon. Identity and parameters: pokemon_data/assets/moves/192.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 120},
  {kind = Effect.Status, status = Status.Paralyzed, chance = 1}
}}

move.id = "zap-cannon"
move.name = "Zap Cannon"
move.type = Type.Electric
move.category = Category.Special
move.pp = 5
move.accuracy = 0.5
move.priority = 0
move.target = Target.Selected
move.flags = {"bullet"}
return move
