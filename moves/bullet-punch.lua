-- Move 418: Bullet Punch. Identity and parameters: pokemon_data/assets/moves/418.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 40}
}}

move.id = "bullet-punch"
move.name = "Bullet Punch"
move.type = Type.Steel
move.category = Category.Physical
move.pp = 30
move.accuracy = 1
move.priority = 1
move.target = Target.Selected
move.flags = {"contact", "punch"}
return move
