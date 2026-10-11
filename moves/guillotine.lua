-- Move 12: Guillotine. Identity and parameters: pokemon_data/assets/moves/12.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {{kind = Effect.Ohko}}}

move.id = "guillotine"
move.name = "Guillotine"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 5
move.accuracy = Accuracy.Ohko
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
