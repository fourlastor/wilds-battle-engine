-- Move 811: Coaching. Identity and parameters: pokemon_data/assets/moves/811.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
local changed = false
for _, pokemon in ipairs(ctx.allies) do
  for _, stat in ipairs({Stat.Attack, Stat.Defense}) do
    changed = ctx:change_stat(pokemon, stat, 1) ~= 0 or changed
  end
end
if not changed then return ctx:fail() end
end}

move.id = "coaching"
move.name = "Coaching"
move.type = Type.Fighting
move.category = Category.Status
move.pp = 10
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"bypass_substitute"}
return move
