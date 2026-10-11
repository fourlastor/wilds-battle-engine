-- Move 441: Gunk Shot. Identity and parameters: pokemon_data/assets/moves/441.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 120},
  {kind = Effect.Status, status = Status.Poisoned, chance = 0.3}
}}

move.id = "gunk-shot"
move.name = "Gunk Shot"
move.type = Type.Poison
move.category = Category.Physical
move.pp = 5
move.accuracy = 0.8
move.priority = 0
move.target = Target.Selected
move.flags = {"sheer_force"}
return move
