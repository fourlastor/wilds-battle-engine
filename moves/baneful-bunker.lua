-- Move 661. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "baneful-bunker", name = "Baneful Bunker", type = Type.Poison,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 4, target = Target.User,
  flags = {},
  script = function(ctx)
    if #ctx.pending == 0 or not ctx:check_protection() then ctx:fail(); return end
    ctx:start_effect(ctx.user, "baneful_bunker", {turns=1, rules={{kind=Rule.Guard, contact_status=Status.Poisoned}}})
  end,
}
