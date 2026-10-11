-- Move 254: Stockpile. No local source JSON; identity and parameters from PokeAPI.
-- Behavior checked against the reference recorded in catalog.json.

return {
  id = "stockpile",
  name = "Stockpile",
  type = Type.Normal,
  category = Category.Status,
  pp = 20,
  accuracy = Accuracy.Always,
  priority = 0,
  target = Target.User,
  flags = {"snatch"},
  script = function(ctx)
    local stored = ctx.user.marks.stockpile or 0
    if stored >= 3 then return ctx:fail() end

    ctx:mark(ctx.user, "stockpile", stored + 1)
    -- Count successful applications; repayment passes through the current ability too.
    local defense = ctx:change_stat(ctx.user, Stat.Defense, 1)
    local special = ctx:change_stat(ctx.user, Stat.SpDefense, 1)
    ctx:mark(ctx.user, "stockpile_defense", (ctx.user.marks.stockpile_defense or 0) + (defense ~= 0 and 1 or 0))
    ctx:mark(ctx.user, "stockpile_sp_defense", (ctx.user.marks.stockpile_sp_defense or 0) + (special ~= 0 and 1 or 0))
    ctx:message(ctx.user.name .. " stockpiled " .. (stored + 1) .. "!")
  end,
}
