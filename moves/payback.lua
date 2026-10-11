-- Move 371: Payback. Identity and parameters: pokemon_data/assets/moves/371.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {flags = {"contact"},
    script = function(ctx) ctx:damage(ctx.target.acted and 100 or 50) end}

move.id = "payback"
move.name = "Payback"
move.type = Type.Dark
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
