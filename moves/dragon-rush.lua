-- Move 407: Dragon Rush. Identity and parameters: pokemon_data/assets/moves/407.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 100},
  {kind = Effect.Flinch, chance = 0.2}
}}

move.id = "dragon-rush"
move.name = "Dragon Rush"
move.type = Type.Dragon
move.category = Category.Physical
move.pp = 10
move.accuracy = 0.75
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
