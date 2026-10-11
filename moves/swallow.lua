-- Move 256: Swallow. Identity and parameters: pokemon_data/assets/moves/256.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Stockpile is outside the sheet catalog; requires compatible stockpile, stockpile_defense and stockpile_sp_defense marks from a host move.

local move = {script = function(ctx)
local stored = ctx.user.marks.stockpile or 0
if stored == 0 then return ctx:fail() end
ctx:heal(ctx.user, {fraction = ({1/4, 1/2, 1})[stored]})
ctx:change_stat(ctx.user, Stat.Defense, -(ctx.user.marks.stockpile_defense or 0))
ctx:change_stat(ctx.user, Stat.SpDefense, -(ctx.user.marks.stockpile_sp_defense or 0))
ctx:unmark(ctx.user, "stockpile")
ctx:unmark(ctx.user, "stockpile_defense")
ctx:unmark(ctx.user, "stockpile_sp_defense")
end}

move.id = "swallow"
move.name = "Swallow"
move.type = Type.Normal
move.category = Category.Status
move.pp = 10
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch", "incomplete"}
return move
