-- Move 195. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "perish-song", name = "Perish Song", type = Type.Normal,
  category = Category.Status, pp = 5, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"sound", "bypass_substitute"},
  script = function(ctx)
    local applied = false
    for _, who in ipairs(ctx.everyone) do
      local ability = (who.ability or ""):gsub("[-_]", "")
      if ability ~= "soundproof" and not who.effects.perish_song and ctx:try_hit(who, {ignore_protect = true}) then
        if ctx:start_effect(who, "perish_song", {turns = 4, rules = {{kind = Rule.FaintOnExpiry}}}) then applied = true end
      end
    end
    if applied then ctx:succeed() else ctx:fail() end
  end,
}
