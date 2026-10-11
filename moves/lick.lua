-- Move 122: Lick. Identity and parameters: pokemon_data/assets/moves/122.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 30},
  {kind = Effect.Status, status = Status.Paralyzed, chance = 0.3}
}}

move.id = "lick"
move.name = "Lick"
move.type = Type.Ghost
move.category = Category.Physical
move.pp = 30
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "sheer_force"}
return move
