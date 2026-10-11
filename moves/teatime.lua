-- Move 752. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "teatime", name = "Teatime", type = Type.Normal,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.Field,
  flags = {"bypass_substitute"},
  script = function(ctx)
    local ate = false
    for _, pokemon in ipairs(ctx.everyone) do
      if ctx:try_hit(pokemon, {ignore_protect = true}) and ctx:eat_berry(pokemon, true) then ate = true end
    end
    if not ate then ctx:message("But it failed!") end
    ctx:succeed()
  end,
}
