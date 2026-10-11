-- Move 713: Sunsteel Strike. Identity and parameters: pokemon_data/assets/moves/713.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 100}
}}

move.id = "sunsteel-strike"
move.name = "Sunsteel Strike"
move.type = Type.Steel
move.category = Category.Physical
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"ignore_ability", "contact"}
return move
