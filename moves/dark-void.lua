-- Move 464: Dark Void. Identity and parameters: pokemon_data/assets/moves/464.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
if ctx.user.species ~= "darkrai" then return ctx:fail() end
for _, pokemon in ipairs(ctx.targets) do
  if ctx:reached(pokemon) then ctx:apply_status(pokemon, Status.Asleep) end
end
end}

move.id = "dark-void"
move.name = "Dark Void"
move.type = Type.Dark
move.category = Category.Status
move.pp = 10
move.accuracy = 0.5
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"reflectable"}
return move
