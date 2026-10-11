-- Move 172: Flame Wheel. Identity and parameters: pokemon_data/assets/moves/172.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 60},
  {kind = Effect.Status, status = Status.Burned, chance = 0.1}
}}

move.id = "flame-wheel"
move.name = "Flame Wheel"
move.type = Type.Fire
move.category = Category.Physical
move.pp = 25
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
move.usable_while_frozen = true
return move
