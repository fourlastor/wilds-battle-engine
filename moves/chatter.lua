-- Move 448: Chatter. Identity and parameters: pokemon_data/assets/moves/448.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
if ctx:damage(65).hit then ctx:confuse(ctx.target) end
end}

move.id = "chatter"
move.name = "Chatter"
move.type = Type.Flying
move.category = Category.Special
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
move.base_power = 65
return move
