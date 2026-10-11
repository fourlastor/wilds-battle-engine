-- Move 824: Glacial Lance. Identity and parameters: pokemon_data/assets/moves/824.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(120, {target = pokemon})
    if hit.hit then
      connected = true
    end
  end
end}

move.id = "glacial-lance"
move.name = "Glacial Lance"
move.type = Type.Ice
move.category = Category.Physical
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.AllOpponents
move.flags = {}
return move
