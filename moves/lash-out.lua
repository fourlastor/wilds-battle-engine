-- Move 808: Lash Out. Identity and parameters: pokemon_data/assets/moves/808.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
ctx:damage(ctx.user.stats_lowered_this_turn and 150 or 75)
end}

move.id = "lash-out"
move.name = "Lash Out"
move.type = Type.Dark
move.category = Category.Physical
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
