-- Move 270. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "helping-hand", name = "Helping Hand", type = Type.Normal,
  category = Category.Status, pp = 20, accuracy = Accuracy.Always,
  priority = 5, target = Target.Ally,
  flags = {"bypass_substitute"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target, {ignore_protect = true}) then return end
    local pending = false
    for _, who in ipairs(ctx.pending) do if who == ctx.target then pending = true; break end end
    if not pending and (ctx.target.acted or ctx.target.turns_on_field ~= 0) then ctx:fail(); return end
    local count = (ctx.target.marks.helping_hand or 0) + 1
    ctx:start_effect(ctx.target, "helping_hand", {turns = 1, restart = true, value = count,
      rules = {{kind = Rule.BasePower, factor = 1.5 ^ count}}})
  end,
}
