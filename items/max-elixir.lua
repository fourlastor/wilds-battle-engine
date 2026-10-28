return {
  id = "max-elixir", name = "Max Elixir", targets = "party",
  usable = function(target)
    for move, pp in pairs(target.pp) do
      if pp < target.max_pp[move] then return true end
    end
    return false
  end,
  script = function(ctx)
    if ctx:restore_pp(ctx.target, nil, nil) > 0 then
      ctx:message("{0}'s PP was restored.", ctx.target)
    end
  end,
}
