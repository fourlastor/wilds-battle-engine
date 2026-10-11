-- Move 210: Fury Cutter. Identity and parameters: pokemon_data/assets/moves/210.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
local uses = ctx.user.last_move == ctx.move.id and ctx.user.streak or 0
ctx:damage(math.min(40 * 2 ^ uses, 160))
end}

move.id = "fury-cutter"
move.name = "Fury Cutter"
move.type = Type.Bug
move.category = Category.Physical
move.pp = 20
move.accuracy = 0.95
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "slicing"}
return move
