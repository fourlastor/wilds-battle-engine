-- Move 117. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "bide", name = "Bide", type = Type.Normal,
  category = Category.Physical, pp = 10, accuracy = Accuracy.Always,
  priority = 1, target = Target.User,
  flags = {"contact"},
  on_hit = function(ctx)
    ctx:mark(ctx.user, "bide_damage", (ctx.user.marks.bide_damage or 0) + ctx.hit.damage)
    ctx:mark(ctx.user, "bide_source", ctx.target.index)
    ctx:mark(ctx.user, "bide_side", ctx.target.side == "allies" and 1 or 2)
  end,
  on_interrupt = function(ctx)
    ctx:unmark(ctx.user, "bide_damage")
    ctx:unmark(ctx.user, "bide_source")
    ctx:unmark(ctx.user, "bide_side")
  end,
  script = function(ctx)
    if ctx.turn == 1 then
      ctx:mark(ctx.user, "bide_damage", 0)
      ctx:unmark(ctx.user, "bide_source")
      ctx:force_move(3, TargetPolicy.SameTarget)
    end
    if ctx.turn < 3 then
      ctx:watch_hits_until_next_action()
      ctx:message(ctx.user.name .. " is storing energy!")
      return
    end
    local damage = ctx.user.marks.bide_damage or 0
    local source = ctx.user.marks.bide_source
    local side = ctx.user.marks.bide_side == 1 and "allies" or "foes"
    local target
    for _, pokemon in ipairs(ctx.everyone) do
      if pokemon.index == source and pokemon.side == side then target = pokemon; break end
    end
    ctx:unmark(ctx.user, "bide_damage")
    ctx:unmark(ctx.user, "bide_source")
    ctx:unmark(ctx.user, "bide_side")
    if damage == 0 then ctx:fail(); return end
    if not target and #ctx.foes > 0 then target = ctx.foes[ctx:random_int(1,#ctx.foes)] end
    if not target then ctx:fail(); return end
    if ctx:try_hit(target) then ctx:direct_damage(target, math.min(65535,2*damage), {typeless=true}) end
  end,
}
