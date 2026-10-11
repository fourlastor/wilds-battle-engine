-- Move 800: Meteor Beam. Identity and parameters: pokemon_data/assets/moves/800.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
if ctx.turn == 1 then
  ctx:change_stat(ctx.user, Stat.SpAttack, 1)
  ctx:message(ctx.user.name .. " is gathering space power!")
  ctx:force_move(2, TargetPolicy.SameTarget)
  return
end
ctx:announce()
ctx:damage(120)
end}

move.id = "meteor-beam"
move.name = "Meteor Beam"
move.type = Type.Rock
move.category = Category.Special
move.pp = 10
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {"charge"}
move.manual_announce = true
move.base_power = 120
return move
