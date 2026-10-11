-- Move 747: Stuff Cheeks. Identity and parameters: pokemon_data/assets/moves/747.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Unavailable: held berries are identifiers only; their activation effects cannot be executed by Lua.

local move = {script = function(ctx)
ctx:message("Stuff Cheeks is unavailable: consuming berries needs engine support.")
return ctx:fail()
end}

move.id = "stuff-cheeks"
move.name = "Stuff Cheeks"
move.type = Type.Normal
move.category = Category.Status
move.pp = 10
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch", "incomplete"}
return move
