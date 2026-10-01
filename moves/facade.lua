return {
  id="facade", name="Facade", type=Type.Normal, category=Category.Physical, pp=20,
  script=function(ctx)
    local status = ctx.user.status
    local boosted = status == ctx.Status.Burned or status == ctx.Status.Paralyzed
      or status == ctx.Status.Poisoned or status == ctx.Status.BadlyPoisoned
    ctx:damage(boosted and 140 or 70)
  end
}
