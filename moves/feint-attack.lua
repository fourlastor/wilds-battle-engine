-- Move 185: Feint Attack. Identity and parameters: pokemon_data/assets/moves/185.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 60}
}}

move.id = "feint-attack"
move.name = "Feint Attack"
move.type = Type.Dark
move.category = Category.Physical
move.pp = 20
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
