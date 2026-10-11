-- Move 81: String Shot. Identity and parameters: pokemon_data/assets/moves/81.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    if ctx:reached(pokemon) then
      connected = true
      ctx:change_stat(pokemon, Stat.Speed, -2)
    end
  end
  if not connected then return ctx:fail() end
end}

move.id = "string-shot"
move.name = "String Shot"
move.type = Type.Bug
move.category = Category.Status
move.pp = 40
move.accuracy = 0.95
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"reflectable"}
return move
