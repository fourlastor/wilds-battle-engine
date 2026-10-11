-- Move 217. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "present", name = "Present", type = Type.Normal,
  category = Category.Physical, pp = 15, accuracy = 0.9,
  priority = 0, target = Target.AnyOther,
  flags = {},
  script = function(ctx)
    local roll = ctx:random_int(0,9)
    if roll < 2 then
      if ctx:healing_hit(ctx.target, math.max(1,math.floor(ctx.target.max_hp/4+0.5))) == 0 then ctx:fail() end
    else
      ctx:damage(roll < 6 and 40 or (roll < 9 and 80 or 120))
    end
  end,
}
