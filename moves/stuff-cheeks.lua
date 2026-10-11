-- Move 747: Stuff Cheeks. Identity and parameters: pokemon_data/assets/moves/747.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
if ctx:change_stat(ctx.user, Stat.Defense, 2) == 0 then return ctx:fail() end
if not ctx:eat_berry(ctx.user, true) then return ctx:fail() end
end}

move.id = "stuff-cheeks"
move.name = "Stuff Cheeks"
move.type = Type.Normal
move.category = Category.Status
move.pp = 10
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch", "requires_held_berry"}
return move
