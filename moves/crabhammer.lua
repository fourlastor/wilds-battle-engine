-- Move 152: Crabhammer. Identity and parameters: pokemon_data/assets/moves/152.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 100, high_crit = true}
}}

move.id = "crabhammer"
move.name = "Crabhammer"
move.type = Type.Water
move.category = Category.Physical
move.pp = 10
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
