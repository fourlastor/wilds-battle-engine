-- Move 350: Rock Blast. Identity and parameters: pokemon_data/assets/moves/350.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)

  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:multi_hit(25, 2, 5, {target = pokemon})
    if hit.hit then
      connected = true
    end
  end
end}

move.id = "rock-blast"
move.name = "Rock Blast"
move.type = Type.Rock
move.category = Category.Physical
move.pp = 10
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {"bullet", "multi_hit"}
move.base_power = 25
return move
