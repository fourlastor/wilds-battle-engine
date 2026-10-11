-- Move 2: Karate Chop. Identity and parameters: pokemon_data/assets/moves/2.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 50, high_crit = true}
}}

move.id = "karate-chop"
move.name = "Karate Chop"
move.type = Type.Fighting
move.category = Category.Physical
move.pp = 25
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
