-- Move 336: Howl. Identity and parameters: pokemon_data/assets/moves/336.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    if ctx:reached(pokemon) then
      connected = true
      ctx:change_stat(pokemon, Stat.Attack, 1)
    end
  end
  if not connected then return ctx:fail() end
end}

move.id = "howl"
move.name = "Howl"
move.type = Type.Normal
move.category = Category.Status
move.pp = 40
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.UserAndAllies
move.flags = {"snatch", "sound"}
return move
