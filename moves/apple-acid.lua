-- Move 787: identity and parameters from pokemon_data/assets/moves/787.json.
return {
  id = "apple-acid", name = "Apple Acid",
  type = Type.Grass, category = Category.Special,
  pp = 10, accuracy = 1, priority = 0, target = Target.Selected,
  script = function(ctx)
    if ctx:damage(80).hit and ctx:reached() then
      ctx:change_stat(ctx.target, Stat.SpDefense, -1)
    end
  end,
}
