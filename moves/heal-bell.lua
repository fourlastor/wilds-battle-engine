-- Move 215. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "heal-bell", name = "Heal Bell", type = Type.Normal,
  category = Category.Status, pp = 5, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"sound", "bypass_substitute", "snatch"},
  script = function(ctx)
    local cured = false
    for _, pokemon in ipairs(ctx.user.team.pokemon) do
      local ability = (pokemon.ability or ""):gsub("[^%w]", ""):lower()
      if pokemon == ctx.user or ability ~= "soundproof" then
        if ctx:cure_status(pokemon) then cured = true end
      end
    end
    if not cured then ctx:fail() end
  end,
}
