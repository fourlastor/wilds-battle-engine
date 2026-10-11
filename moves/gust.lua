-- Move 16: Gust. Identity and parameters: pokemon_data/assets/moves/16.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {hits_hidden = {Hidden.Air},
    script = function(ctx) ctx:damage(ctx.target.hidden == Hidden.Air and 80 or 40) end}

move.id = "gust"
move.name = "Gust"
move.type = Type.Flying
move.category = Category.Special
move.pp = 35
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"wind"}
move.base_power = 40
return move
