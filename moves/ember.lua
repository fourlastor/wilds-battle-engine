-- Move 52: Ember. Identity and parameters: pokemon_data/assets/moves/52.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 40},
  {kind = Effect.Status, status = Status.Burned, chance = 0.1}
}}

move.id = "ember"
move.name = "Ember"
move.type = Type.Fire
move.category = Category.Special
move.pp = 25
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"sheer_force"}
return move
