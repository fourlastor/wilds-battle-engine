return {
  id = "soda-pop", name = "Soda Pop", targets = "party",
  usable = function(target) return target.hp < target.max_hp end,
  script = function(ctx)
    local healed = ctx:heal(ctx.target, 50)
    if healed > 0 then ctx:message("{0}'s HP was restored.", ctx.target) end
  end,
}
