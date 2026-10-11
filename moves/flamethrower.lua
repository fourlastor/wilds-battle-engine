-- Move 53: Flamethrower. Identity and parameters: pokemon_data/assets/moves/53.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 90},
  {kind = Effect.Status, status = Status.Burned, chance = 0.1}
}}

move.id = "flamethrower"
move.name = "Flamethrower"
move.type = Type.Fire
move.category = Category.Special
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
