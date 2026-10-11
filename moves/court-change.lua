-- Move 756. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "court-change", name = "Court Change", type = Type.Normal,
  category = Category.Status, pp = 10, accuracy = 1.0,
  priority = 0, target = Target.User,
  flags = {"pressure_all"},
  script = function(ctx)
    if not ctx:swap_side_effects({"mist", "light_screen", "reflect", "spikes", "safeguard", "tailwind", "toxic_spikes", "stealth_rock", "rainbow", "sea_of_fire", "swamp", "sticky_web", "aurora_veil", "lucky_chant"}) then ctx:fail() end
  end,
}
