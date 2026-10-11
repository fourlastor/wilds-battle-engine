-- Move 735: Sizzly Slide. Identity and parameters: pokemon_data/assets/moves/735.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 60},
  {kind = Effect.Status, status = Status.Burned, chance = 1}
}}

move.id = "sizzly-slide"
move.name = "Sizzly Slide"
move.type = Type.Fire
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "sheer_force"}
return move
