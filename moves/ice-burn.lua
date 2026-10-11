-- Move 554: Ice Burn. Identity and parameters: pokemon_data/assets/moves/554.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
if ctx.turn == 1 then
  ctx:message(ctx.user.name .. " is charging!")
  ctx:force_move(2, TargetPolicy.SameTarget)
  return
end
ctx:announce()
if ctx:damage(140).hit then ctx:apply_status(ctx.target, Status.Burned, {chance = 0.3}) end
end}

move.id = "ice-burn"
move.name = "Ice Burn"
move.type = Type.Ice
move.category = Category.Special
move.pp = 5
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {"charge"}
move.manual_announce = true
move.base_power = 140
return move
