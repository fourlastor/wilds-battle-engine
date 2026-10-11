-- Move 243. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "mirror-coat", name = "Mirror Coat", type = Type.Psychic,
  category = Category.Special, pp = 20, accuracy = 1.0,
  priority = -5, target = Target.AnyOther,
  flags = {},
  on_turn_start = function(ctx)
    ctx:unmark(ctx.user, "mirror_coat_damage")
    ctx:unmark(ctx.user, "mirror_coat_slot")
    ctx:watch_hits_until_next_action()
  end,
  on_hit = function(ctx)
    if ctx.target.is_foe and ctx.hit.category == Category.Special and ctx.hit.damage > 0 then
      ctx:mark(ctx.user, "mirror_coat_damage", ctx.hit.damage, 1)
      ctx:mark(ctx.user, "mirror_coat_slot", ctx.target.position, 1)
    end
  end,
  script = function(ctx)
    local damage = ctx.user.marks.mirror_coat_damage or 0
    local target
    for _, pokemon in ipairs(ctx.foes) do
      if pokemon.position == ctx.user.marks.mirror_coat_slot then target = pokemon; break end
    end
    if damage == 0 or not target then return ctx:fail() end
    target = ctx:redirect_target(target)
    if ctx:try_hit(target) then ctx:direct_damage(target, math.min(65535, math.floor(damage * 2.0))) end
  end,
}
