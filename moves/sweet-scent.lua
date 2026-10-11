-- Move 230: Sweet Scent. Identity and parameters: pokemon_data/assets/moves/230.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    if ctx:reached(pokemon) then
      connected = true
      ctx:change_stat(pokemon, Stat.Evasion, -2)
    end
  end
  if not connected then return ctx:fail() end
end}

move.id = "sweet-scent"
move.name = "Sweet Scent"
move.type = Type.Normal
move.category = Category.Status
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"reflectable"}
return move
