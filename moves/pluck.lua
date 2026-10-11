-- Move 365: Pluck. Identity and parameters: pokemon_data/assets/moves/365.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Berry consumption effects and history are implemented; resistance and reactive berry timing still require engine support.

local move = {script = function(ctx)
local hit = ctx:damage(60)
if hit.hit then ctx:eat_stolen_berry(ctx.target, ctx.user) end
end}

move.id = "pluck"
move.name = "Pluck"
move.type = Type.Flying
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "incomplete"}
return move
