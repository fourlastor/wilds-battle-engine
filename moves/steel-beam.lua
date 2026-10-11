-- Move 796: Steel Beam. Identity and parameters: pokemon_data/assets/moves/796.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
ctx:damage(140)
ctx:hurt(ctx.user, math.ceil(ctx.user.max_hp / 2))
end}

move.id = "steel-beam"
move.name = "Steel Beam"
move.type = Type.Steel
move.category = Category.Special
move.pp = 5
move.accuracy = 0.95
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
