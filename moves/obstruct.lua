-- Move 792. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "obstruct", name = "Obstruct", type = Type.Dark,
  category = Category.Status, pp = 10, accuracy = 1.0,
  priority = 4, target = Target.User,
  flags = {},
  script = function(ctx)
    if #ctx.pending == 0 or not ctx:check_protection() then ctx:fail(); return end
    ctx:start_effect(ctx.user, "obstruct", {turns=1, rules={{kind=Rule.Guard, damaging_only=true, contact_stat=Stat.Defense, contact_stages=-2}}})
  end,
}
