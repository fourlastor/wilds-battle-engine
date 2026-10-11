-- Move 413: Brave Bird. Identity and parameters: pokemon_data/assets/moves/413.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 120, recoil = 0.33}
}}

move.id = "brave-bird"
move.name = "Brave Bird"
move.type = Type.Flying
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
