-- Move 575: Parting Shot. Identity and parameters: pokemon_data/assets/moves/575.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  if not ctx:try_hit(ctx.target) then return end
  local attack = ctx:change_stat(ctx.target, Stat.Attack, -1)
  local special = ctx:change_stat(ctx.target, Stat.SpAttack, -1)
  if attack == 0 and special == 0 then return ctx:fail() end
  ctx:switch_out()
end}

move.id = "parting-shot"
move.name = "Parting Shot"
move.type = Type.Dark
move.category = Category.Status
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"reflectable", "sound"}
return move
