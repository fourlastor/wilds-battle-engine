-- Move 424: Fire Fang. Identity and parameters: pokemon_data/assets/moves/424.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 65},
  {kind = Effect.Status, status = Status.Burned, chance = 0.1},
  {kind = Effect.Flinch, chance = 0.1}
}}

move.id = "fire-fang"
move.name = "Fire Fang"
move.type = Type.Fire
move.category = Category.Physical
move.pp = 15
move.accuracy = 0.95
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "bite", "sheer_force"}
return move
