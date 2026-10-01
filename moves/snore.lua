return {
  id="snore", name="Snore", type=Type.Normal, category=Category.Special, pp=15,
  usable_while_asleep=true,
  script=function(ctx)
    if ctx.user.status ~= ctx.Status.Asleep then return ctx:fail() end
    local result = ctx:damage(50)
    if result.hit then ctx:flinch_target(0.3) end
  end
}
