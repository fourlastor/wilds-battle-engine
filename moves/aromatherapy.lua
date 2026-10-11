-- Move 312. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "aromatherapy", name = "Aromatherapy", type = Type.Grass,
  category = Category.Status, pp = 5, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"snatch"},
  script = function(ctx)
    local cured = false
    for _, pokemon in ipairs(ctx.user.team.pokemon) do
      local ability = (pokemon.ability or ""):gsub("[^%w]", ""):lower()
      if pokemon == ctx.user or ability ~= "sapsipper" then
        if ctx:cure_status(pokemon) then cured = true end
      end
    end
    if not cured then ctx:fail() end
  end,
}
