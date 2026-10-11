-- Move 208: Milk Drink. Identity and parameters: pokemon_data/assets/moves/208.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {fail_on_full_hp = true, effects = {
  {kind = Effect.Heal, fraction = 0.5}
}}

move.id = "milk-drink"
move.name = "Milk Drink"
move.type = Type.Normal
move.category = Category.Status
move.pp = 5
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch"}
return move
