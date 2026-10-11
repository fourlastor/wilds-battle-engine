-- Move 754: identity and parameters from pokemon_data/assets/moves/754.json.
return {
  id = "bolt-beak", name = "Bolt Beak",
  type = Type.Electric, category = Category.Physical,
  pp = 10, accuracy = 1, priority = 0, target = Target.Selected,
  flags = {"contact"},
  script = function(ctx)
    local first = ctx.target.turns_on_field == 0
    for _, pokemon in ipairs(ctx.pending) do
      if pokemon == ctx.target then first = true end
    end
    ctx:damage(first and 170 or 85)
  end,
}
