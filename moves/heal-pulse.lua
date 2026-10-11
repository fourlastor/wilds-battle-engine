-- Move 505: Heal Pulse. Identity and parameters: pokemon_data/assets/moves/505.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
if ctx.target.hp == ctx.target.max_hp then return ctx:fail() end
if ctx:reached() then ctx:heal(ctx.target, {fraction = 1/2}) end
end}

move.id = "heal-pulse"
move.name = "Heal Pulse"
move.type = Type.Psychic
move.category = Category.Status
move.pp = 10
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.AnyOther
move.flags = {"reflectable", "pulse"}
return move
