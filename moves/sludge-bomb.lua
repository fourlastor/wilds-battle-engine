-- Move 188: Sludge Bomb. Identity and parameters: pokemon_data/assets/moves/188.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 90},
  {kind = Effect.Status, status = Status.Poisoned, chance = 0.3}
}}

move.id = "sludge-bomb"
move.name = "Sludge Bomb"
move.type = Type.Poison
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"bullet"}
return move
