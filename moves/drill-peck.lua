-- Move 65: Drill Peck. Identity and parameters: pokemon_data/assets/moves/65.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80}
}}

move.id = "drill-peck"
move.name = "Drill Peck"
move.type = Type.Flying
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
