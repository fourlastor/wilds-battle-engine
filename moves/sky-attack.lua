-- Move 143: Sky Attack. Identity and parameters: pokemon_data/assets/moves/143.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
if ctx.turn == 1 then
  ctx:message(ctx.user.name .. " became cloaked in a harsh light!")
  ctx:force_move(2, TargetPolicy.SameTarget)
  return
end
ctx:announce()
if ctx:damage(140, {high_crit = true}).hit then ctx:flinch(ctx.target, {chance = 0.3}) end
end}

move.id = "sky-attack"
move.name = "Sky Attack"
move.type = Type.Flying
move.category = Category.Physical
move.pp = 5
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {"charge", "flinch_secondary"}
move.manual_announce = true
move.base_power = 140
return move
