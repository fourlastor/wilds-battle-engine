-- Move 512: Acrobatics. Identity and parameters: pokemon_data/assets/moves/512.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {flags = {"contact"},
    script = function(ctx) ctx:damage(ctx.user.item and 55 or 110) end}

move.id = "acrobatics"
move.name = "Acrobatics"
move.type = Type.Flying
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
