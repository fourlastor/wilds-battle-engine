-- Move 209: Spark. Identity and parameters: pokemon_data/assets/moves/209.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 65},
  {kind = Effect.Status, status = Status.Paralyzed, chance = 0.3}
}}

move.id = "spark"
move.name = "Spark"
move.type = Type.Electric
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
