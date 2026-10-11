-- Move 475: Autotomize. Identity and parameters: pokemon_data/assets/moves/475.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
if ctx:change_stat(ctx.user, Stat.Speed, 2) == 0 then return ctx:fail() end
ctx:set_weight(ctx.user, math.max(0.1, ctx.user.weight - 100))
end}

move.id = "autotomize"
move.name = "Autotomize"
move.type = Type.Steel
move.category = Category.Status
move.pp = 15
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch"}
return move
