return {
  base_power = 50,
  flags={"sound"},
  id="snore", name="Snore", type=Type.Normal, category=Category.Special, pp=15,
  usable_while_asleep=true,
  script=function(ctx)
    if not ctx.user.asleep then return ctx:fail() end
    local result = ctx:damage(50)
    if result.hit then ctx:flinch_target(0.3) end
  end
}
