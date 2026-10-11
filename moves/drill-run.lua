-- Move 529: Drill Run. Identity and parameters: pokemon_data/assets/moves/529.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80, high_crit = true}
}}

move.id = "drill-run"
move.name = "Drill Run"
move.type = Type.Ground
move.category = Category.Physical
move.pp = 10
move.accuracy = 0.95
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
