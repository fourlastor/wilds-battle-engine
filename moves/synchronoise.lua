-- Move 485: Synchronoise. Identity and parameters: pokemon_data/assets/moves/485.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
local hit = false
for _, pokemon in ipairs(ctx.targets) do
  if ctx:share_type(ctx.user, pokemon) then hit = ctx:damage(120, {target = pokemon}).hit or hit end
end
if not hit then return ctx:fail() end
end}

move.id = "synchronoise"
move.name = "Synchronoise"
move.type = Type.Psychic
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.AllOthers
move.flags = {}
return move
