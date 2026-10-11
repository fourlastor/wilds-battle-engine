-- Move 506: Hex. Identity and parameters: pokemon_data/assets/moves/506.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
    script = function(ctx) ctx:damage(ctx.target.status and 130 or 65) end}

move.id = "hex"
move.name = "Hex"
move.type = Type.Ghost
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
