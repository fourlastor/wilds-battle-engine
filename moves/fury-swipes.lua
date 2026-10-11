-- Move 154: Fury Swipes. Identity and parameters: pokemon_data/assets/moves/154.json.
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

move.id = "fury-swipes"
move.name = "Fury Swipes"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 15
move.accuracy = 0.8
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
move.base_power = 18
return move
