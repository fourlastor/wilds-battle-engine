-- Move 599: Venom Drench. Identity and parameters: pokemon_data/assets/moves/599.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
local affected = false
for _, pokemon in ipairs(ctx.targets) do
  if (pokemon.status == Status.Poisoned or pokemon.status == Status.BadlyPoisoned) and ctx:reached(pokemon) then
    for _, stat in ipairs({Stat.Attack, Stat.SpAttack, Stat.Speed}) do
      affected = ctx:change_stat(pokemon, stat, -1) ~= 0 or affected
    end
  end
end
if not affected then return ctx:fail() end
end}

move.id = "venom-drench"
move.name = "Venom Drench"
move.type = Type.Poison
move.category = Category.Status
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"reflectable"}
return move
