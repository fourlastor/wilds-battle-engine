-- Move 507: Sky Drop. Identity and parameters: pokemon_data/assets/moves/507.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  if ctx.turn == 1 then
    if not ctx:try_hit(ctx.target, {never_miss = true}) then return end
    if not ctx:carry(ctx.target) then return ctx:fail() end
    ctx:message("{0} lifted {1} into the air!", ctx.user, ctx.target)
    ctx:force_move(2, TargetPolicy.SameTarget)
    return
  end
  if not ctx:release() then return ctx:fail() end
  ctx:damage(60, {effective = {[Type.Flying] = 0}})
end}

move.id = "sky-drop"
move.name = "Sky Drop"
move.type = Type.Flying
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"gravity", "charge", "no_power_herb", "contact"}
move.base_power = 60
return move
