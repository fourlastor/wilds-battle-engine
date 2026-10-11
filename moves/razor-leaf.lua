-- Move 75: Razor Leaf. Identity and parameters: pokemon_data/assets/moves/75.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(55, {target = pokemon, high_crit = true})
    if hit.hit then
      connected = true
    end
  end
end}

move.id = "razor-leaf"
move.name = "Razor Leaf"
move.type = Type.Grass
move.category = Category.Physical
move.pp = 25
move.accuracy = 0.95
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"slicing"}
move.base_power = 55
return move
