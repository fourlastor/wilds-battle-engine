-- Move 352: Water Pulse. Identity and parameters: pokemon_data/assets/moves/352.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(60, {target = pokemon})
    if hit.hit then
      connected = true
      ctx:confuse(pokemon, {chance = 0.2})
    end
  end
end}

move.id = "water-pulse"
move.name = "Water Pulse"
move.type = Type.Water
move.category = Category.Special
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"pulse"}
move.base_power = 60
return move
