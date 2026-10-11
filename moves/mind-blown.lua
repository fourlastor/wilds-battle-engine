-- Move 720: Mind Blown. Identity and parameters: pokemon_data/assets/moves/720.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
ctx:damage(150)
if ctx.last_hit then ctx:hurt(ctx.user, math.ceil(ctx.user.max_hp / 2)) end
end}

move.id = "mind-blown"
move.name = "Mind Blown"
move.type = Type.Fire
move.category = Category.Special
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.AllOthers
move.flags = {}
move.base_power = 150
return move
