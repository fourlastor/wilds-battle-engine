-- Move 517: Inferno. Identity and parameters: pokemon_data/assets/moves/517.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 100},
  {kind = Effect.Status, status = Status.Burned, chance = 1}
}}

move.id = "inferno"
move.name = "Inferno"
move.type = Type.Fire
move.category = Category.Special
move.pp = 5
move.accuracy = 0.5
move.priority = 0
move.target = Target.Selected
move.flags = {"sheer_force"}
return move
