-- Move 423: Ice Fang. Identity and parameters: pokemon_data/assets/moves/423.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 65},
  {kind = Effect.Status, status = Status.Frozen, chance = 0.1},
  {kind = Effect.Flinch, chance = 0.1}
}}

move.id = "ice-fang"
move.name = "Ice Fang"
move.type = Type.Ice
move.category = Category.Physical
move.pp = 15
move.accuracy = 0.95
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "bite", "sheer_force"}
return move
