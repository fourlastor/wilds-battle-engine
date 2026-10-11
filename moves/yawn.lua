-- Move 281. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "yawn", name = "Yawn", type = Type.Normal,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {"reflectable"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    if ctx.field.effects.electric_terrain and ctx.target.on_ground and not ctx.target.hidden then ctx:fail(); return end
    if not ctx:can_apply_status(ctx.target, Status.Asleep, {ignore_terrain = true}) then ctx:fail(); return end
    if not ctx:start_effect(ctx.target, "yawn", {turns = 2, rules = {
      {kind = Rule.StatusOnExpiry, status = Status.Asleep, by = ctx.user}
    }}) then ctx:fail() end
  end,
}
