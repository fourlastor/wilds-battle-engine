-- Move 801: Shell Side Arm. Identity and parameters: pokemon_data/assets/moves/801.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
local physical = ctx.user.attack / math.max(1, ctx.target.defense)
local special = ctx.user.sp_attack / math.max(1, ctx.target.sp_defense)
local use_physical = physical > special or (physical == special and ctx:chance(0.5))
local hit = ctx:damage(90, {category = use_physical and Category.Physical or Category.Special, contact = use_physical})
if hit.hit then ctx:apply_status(ctx.target, Status.Poisoned, {chance = 0.2}) end
end}

move.id = "shell-side-arm"
move.name = "Shell Side Arm"
move.type = Type.Poison
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
move.base_power = 90
return move
