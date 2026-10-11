-- Move 4: Comet Punch. Identity and parameters: pokemon_data/assets/moves/4.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)

  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:multi_hit(18, 2, 5, {target = pokemon})
    if hit.hit then
      connected = true
    end
  end
end}

move.id = "comet-punch"
move.name = "Comet Punch"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 15
move.accuracy = 0.85
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "punch"}
move.base_power = 18
return move
