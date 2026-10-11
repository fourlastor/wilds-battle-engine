-- Move 810. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "corrosive-gas", name = "Corrosive Gas", type = Type.Poison,
  category = Category.Status, pp = 40, accuracy = 1.0,
  priority = 0, target = Target.AllOthers,
  flags = {"reflectable"},
  script = function(ctx)
    local removed = false
    for _, pokemon in ipairs(ctx.targets) do
      if ctx:adjacent(ctx.user, pokemon) and ctx:try_hit(pokemon) then
        if ctx:take_item(pokemon) then removed = true end
      end
    end
    if removed then ctx:succeed() else ctx:fail() end
  end,
}
