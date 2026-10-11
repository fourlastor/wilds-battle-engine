-- Move 168: Thief. Identity and parameters: pokemon_data/assets/moves/168.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
if not ctx:damage(60).hit then return end
ctx:steal_item(ctx.target, ctx.user)
end}

move.id = "thief"
move.name = "Thief"
move.type = Type.Dark
move.category = Category.Physical
move.pp = 25
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
move.base_power = 60
return move
