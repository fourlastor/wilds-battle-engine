-- Move 314: Air Cutter. Identity and parameters: pokemon_data/assets/moves/314.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(60, {target = pokemon, high_crit = true})
    if hit.hit then
      connected = true
    end
  end
end}

move.id = "air-cutter"
move.name = "Air Cutter"
move.type = Type.Flying
move.category = Category.Special
move.pp = 25
move.accuracy = 0.95
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"slicing", "wind"}
move.base_power = 60
return move
