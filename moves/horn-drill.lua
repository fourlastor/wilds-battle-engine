-- Move 32: Horn Drill. Identity and parameters: pokemon_data/assets/moves/32.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {{kind = Effect.Ohko}}}

move.id = "horn-drill"
move.name = "Horn Drill"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 5
move.accuracy = Accuracy.Ohko
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
