-- Move 553: Freeze Shock. Identity and parameters: pokemon_data/assets/moves/553.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
if ctx.turn == 1 then
  ctx:message("{0} is charging!", ctx.user)
  ctx:force_move(2, TargetPolicy.SameTarget)
  return
end
ctx:announce()
if ctx:damage(140).hit then ctx:apply_status(ctx.target, Status.Paralyzed, {chance = 0.3}) end
end}

move.id = "freeze-shock"
move.name = "Freeze Shock"
move.type = Type.Ice
move.category = Category.Physical
move.pp = 5
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {"charge", "sheer_force"}
move.manual_announce = true
move.base_power = 140
return move
