-- Move 226. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "baton-pass", name = "Baton Pass", type = Type.Normal,
  category = Category.Status, pp = 40, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"pass_volatiles"},
  script = function(ctx)
    if not ctx:baton_pass({"aqua_ring", "curse", "embargo", "endure", "focus_energy", "heal_block", "helping_hand", "ingrain", "laser_focus", "leech_seed", "magnet_rise", "no_retreat", "octolock", "perish_song", "power_trick", "tar_shot", "taunt", "telekinesis", "throat_chop", "magic_coat", "powder", "roosting", "snatch", "baneful_bunker", "spiky_shield", "kings_shield", "obstruct", "me_first"}) then ctx:fail() end
  end,
}
