-- Move 257: Heat Wave. Identity and parameters: pokemon_data/assets/moves/257.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(95, {target = pokemon})
    if hit.hit then
      connected = true
      ctx:apply_status(pokemon, Status.Burned, {chance = 0.1})
    end
  end
end}

move.id = "heat-wave"
move.name = "Heat Wave"
move.type = Type.Fire
move.category = Category.Special
move.pp = 10
move.accuracy = 0.9
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"wind"}
move.base_power = 95
return move
