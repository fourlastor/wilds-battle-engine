-- Move 684: Smart Strike. Identity and parameters: pokemon_data/assets/moves/684.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 70}
}}

move.id = "smart-strike"
move.name = "Smart Strike"
move.type = Type.Steel
move.category = Category.Physical
move.pp = 10
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
