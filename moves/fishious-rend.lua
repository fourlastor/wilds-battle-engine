-- Move 755: identity and parameters from pokemon_data/assets/moves/755.json.
return {
  id = "fishious-rend", name = "Fishious Rend",
  type = Type.Water, category = Category.Physical,
  pp = 10, accuracy = 1, priority = 0, target = Target.Selected,
  flags = {"contact", "bite"},
  script = function(ctx)
    local first = ctx.target.turns_on_field == 0
    for _, pokemon in ipairs(ctx.pending) do
      if pokemon == ctx.target then first = true end
    end
    ctx:damage(first and 170 or 85)
  end,
}
