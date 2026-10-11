-- Move 730: Splishy Splash. Identity and parameters: pokemon_data/assets/moves/730.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(90, {target = pokemon})
    if hit.hit then
      connected = true
      ctx:apply_status(pokemon, Status.Paralyzed, {chance = 0.3})
    end
  end
end}

move.id = "splishy-splash"
move.name = "Splishy Splash"
move.type = Type.Water
move.category = Category.Special
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"sheer_force"}
move.base_power = 90
return move
