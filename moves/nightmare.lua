-- Move 171. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "nightmare", name = "Nightmare", type = Type.Ghost,
  category = Category.Status, pp = 15, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    local ability = ctx.target.ability and ctx.target.ability:gsub("[-_]", "")
    if (ctx.target.status ~= Status.Asleep and ability ~= "comatose") or ctx.target.effects.nightmare then ctx:fail(); return end
    ctx:start_effect(ctx.target, "nightmare", {rules = {
      {kind = Rule.RequiresStatus, status = Status.Asleep},
      {kind = Rule.DamageEachTurn, fraction = 1/4, message = "{name} is locked in a nightmare!"}
    }})
  end,
}
