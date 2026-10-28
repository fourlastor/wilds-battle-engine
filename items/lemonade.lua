return {
  id = "lemonade", name = "Lemonade", targets = "party",
  usable = function(target) return target.hp < target.max_hp end,
  script = function(ctx)
    local healed = ctx:heal(ctx.target, 70)
    if healed > 0 then ctx:message("{0}'s HP was restored.", ctx.target) end
  end,
}
