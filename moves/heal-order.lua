-- Move 456: Heal Order. Identity and parameters: pokemon_data/assets/moves/456.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {fail_on_full_hp = true, effects = {
  {kind = Effect.Heal, fraction = 0.5}
}}

move.id = "heal-order"
move.name = "Heal Order"
move.type = Type.Bug
move.category = Category.Status
move.pp = 10
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch"}
return move
