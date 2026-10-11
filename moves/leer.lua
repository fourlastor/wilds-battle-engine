-- Move 43: Leer. Identity and parameters: pokemon_data/assets/moves/43.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    if ctx:reached(pokemon) then
      connected = true
      ctx:change_stat(pokemon, Stat.Defense, -1)
    end
  end
  if not connected then return ctx:fail() end
end}

move.id = "leer"
move.name = "Leer"
move.type = Type.Normal
move.category = Category.Status
move.pp = 30
move.accuracy = 1
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"reflectable"}
return move
