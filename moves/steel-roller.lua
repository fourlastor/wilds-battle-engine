-- Move 798: Steel Roller. Identity and parameters: pokemon_data/assets/moves/798.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
local present = false
for _, name in ipairs({"electric_terrain", "grassy_terrain", "misty_terrain", "psychic_terrain"}) do
  present = present or ctx.field.effects[name] ~= nil
end
if not present then return ctx:fail() end
if ctx:damage(130).hit then ctx:end_group(ctx.field, "terrain") end
end}

move.id = "steel-roller"
move.name = "Steel Roller"
move.type = Type.Steel
move.category = Category.Physical
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
move.base_power = 130
return move
