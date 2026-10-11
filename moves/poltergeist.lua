-- Move 809: Poltergeist. Identity and parameters: pokemon_data/assets/moves/809.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
if ctx.hit_index == 1 and not ctx.target.item then return ctx:fail() end
if ctx.hit_index == 1 then ctx:message(ctx.target.name .. " is about to be attacked by its " .. ctx.target.item .. "!") end
ctx:damage(110)
end}

move.id = "poltergeist"
move.name = "Poltergeist"
move.type = Type.Ghost
move.category = Category.Physical
move.pp = 5
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {}
move.base_power = 110
return move
