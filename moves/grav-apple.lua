-- Move 788: Grav Apple. Identity and parameters: pokemon_data/assets/moves/788.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local function gravity(ctx)
  return ctx.field.effects.gravity ~= nil
end

local move = {script = function(ctx)
local power = ctx.field.effects.gravity and 120 or 80
if ctx:damage(power).hit then ctx:change_stat(ctx.target, Stat.Defense, -1) end
end}

move.id = "grav-apple"
move.name = "Grav Apple"
move.type = Type.Grass
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"sheer_force"}
return move
