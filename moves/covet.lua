-- Move 343: Covet. Identity and parameters: pokemon_data/assets/moves/343.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
if not ctx:damage(60).hit then return end
ctx:steal_item(ctx.target, ctx.user)
end}

move.id = "covet"
move.name = "Covet"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 25
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
move.base_power = 60
return move
