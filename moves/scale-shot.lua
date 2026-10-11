-- Move 799: Scale Shot. Identity and parameters: pokemon_data/assets/moves/799.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
local roll = ctx:random_int(1, 100)
local hits = roll <= 35 and 2 or roll <= 70 and 3 or roll <= 85 and 4 or 5
if ctx:multi_hit(25, hits, hits).hit then
  ctx:change_stat(ctx.user, Stat.Defense, -1)
  ctx:change_stat(ctx.user, Stat.Speed, 1)
end
end}

move.id = "scale-shot"
move.name = "Scale Shot"
move.type = Type.Dragon
move.category = Category.Physical
move.pp = 20
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
