local forbidden = {assist=true, beakblast=true, belch=true, bide=true, blastburn=true, blazingtorque=true, bounce=true, celebrate=true, chatter=true, combattorque=true, copycat=true, dig=true, dive=true, dynamaxcannon=true, eternabeam=true, fly=true, focuspunch=true, freezeshock=true, frenzyplant=true, geomancy=true, gigaimpact=true, holdhands=true, hydrocannon=true, hyperbeam=true, iceball=true, iceburn=true, instruct=true, kingsshield=true, magicaltorque=true, mefirst=true, meteorassault=true, metronome=true, mimic=true, mirrormove=true, naturepower=true, noxioustorque=true, obstruct=true, outrage=true, petaldance=true, phantomforce=true, prismaticlaser=true, razorwind=true, roaroftime=true, rockwrecker=true, rollout=true, shadowforce=true, shelltrap=true, sketch=true, skullbash=true, skyattack=true, skydrop=true, sleeptalk=true, solarbeam=true, solarblade=true, struggle=true, thrash=true, transform=true, uproar=true, wickedtorque=true}
-- Move 689. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "instruct", name = "Instruct", type = Type.Psychic,
  category = Category.Status, pp = 15, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {"bypass_substitute"},
  script = function(ctx)
    local target = ctx.target
    local id = target.last_move
    local info = id and ctx:move_info(id)
    if not info or forbidden[id:gsub("[^%w]", ""):lower()] or info.flags.charge or info.flags.recharge or target.recharging or target.pp[id] == 0 then ctx:fail(); return end
    for _, pending in ipairs(ctx.pending) do
      if pending == target and pending.selected_move then
        local selected = pending.selected_move.id:gsub("[^%w]", ""):lower()
        if selected == "beakblast" or selected == "focuspunch" or selected == "shelltrap" then ctx:fail(); return end
      end
    end
    if not ctx:try_hit(target) then return end
    if not ctx:instruct(target) then ctx:fail() end
  end,
}
