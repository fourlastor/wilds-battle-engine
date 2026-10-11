-- Move 139: Poison Gas. Identity and parameters: pokemon_data/assets/moves/139.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    if ctx:reached(pokemon) then
      connected = true
      ctx:apply_status(pokemon, Status.Poisoned, {chance = 1})
    end
  end
  if not connected then return ctx:fail() end
end}

move.id = "poison-gas"
move.name = "Poison Gas"
move.type = Type.Poison
move.category = Category.Status
move.pp = 40
move.accuracy = 0.9
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"reflectable"}
return move
