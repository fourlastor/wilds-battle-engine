-- Move 484. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "heavy-slam", name = "Heavy Slam", type = Type.Steel,
  category = Category.Physical, pp = 10, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"contact"},
  script = function(ctx)
    local w = math.max(0.1, ctx.target.weight)
    local own = math.max(0.1, ctx.user.weight)
    local power = own >= w * 5 and 120 or own >= w * 4 and 100 or own >= w * 3 and 80 or own >= w * 2 and 60 or 40
    local small = ctx.target.marks.minimized ~= nil
    ctx:damage(power * (small and 2 or 1), {never_miss = small})
  end,
}
