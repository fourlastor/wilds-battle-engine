-- Move 616: Land’s Wrath. Identity and parameters: pokemon_data/assets/moves/616.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(90, {target = pokemon})
    if hit.hit then
      connected = true
    end
  end
end}

move.id = "lands-wrath"
move.name = "Land’s Wrath"
move.type = Type.Ground
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.AllOpponents
move.flags = {}
move.base_power = 90
return move
