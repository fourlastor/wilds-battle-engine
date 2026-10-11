-- Move 395: Force Palm. Identity and parameters: pokemon_data/assets/moves/395.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 60},
  {kind = Effect.Status, status = Status.Paralyzed, chance = 0.3}
}}

move.id = "force-palm"
move.name = "Force Palm"
move.type = Type.Fighting
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
