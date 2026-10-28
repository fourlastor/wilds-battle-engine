return {
  id = "fresh-water", name = "Fresh Water", targets = "party",
  usable = function(target) return target.hp < target.max_hp end,
  script = function(ctx)
    local healed = ctx:heal(ctx.target, 30)
    if healed > 0 then ctx:message("{0}'s HP was restored.", ctx.target) end
  end,
}
