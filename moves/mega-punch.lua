-- Move 5: Mega Punch. Identity and parameters: pokemon_data/assets/moves/5.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80}
}}

move.id = "mega-punch"
move.name = "Mega Punch"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 20
move.accuracy = 0.85
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "punch"}
return move
