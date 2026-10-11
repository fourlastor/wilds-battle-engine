-- Move 660: First Impression. Identity and parameters: pokemon_data/assets/moves/660.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
if ctx.user.turns_on_field > 1 then return ctx:fail() end
ctx:damage(90)
end}

move.id = "first-impression"
move.name = "First Impression"
move.type = Type.Bug
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 2
move.target = Target.Selected
move.flags = {"contact"}
move.base_power = 90
return move
