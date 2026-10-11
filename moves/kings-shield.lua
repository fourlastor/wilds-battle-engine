-- Move 588. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "kings-shield", name = "King’s Shield", type = Type.Steel,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 4, target = Target.User,
  flags = {},
  script = function(ctx)
    if #ctx.pending == 0 or not ctx:check_protection() then ctx:fail(); return end
    ctx:start_effect(ctx.user, "kings_shield", {turns=1, rules={{kind=Rule.Guard, damaging_only=true, contact_stat=Stat.Attack, contact_stages=-1}}})
  end,
}
