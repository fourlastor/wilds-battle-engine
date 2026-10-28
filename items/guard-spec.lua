return {
  id = "guard-spec", name = "Guard Spec.", targets = "user",
  script = function(ctx)
    if ctx:start_effect(ctx.target.team, "mist", {turns = 5, rules = {{kind = Rule.BlockStatDrops}}}) then
      ctx:message("{0}'s team became shrouded in mist!", ctx.target)
    else
      ctx:message("But it failed!")
    end
  end,
}
