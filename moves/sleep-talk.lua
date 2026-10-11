-- Gen 8 calling restrictions from Pokemon Showdown 942cda71.
local forbidden = {assist=true, beakblast=true, belch=true, bide=true, blazingtorque=true, bounce=true, celebrate=true, chatter=true, combattorque=true, copycat=true, dig=true, dive=true, dynamaxcannon=true, fly=true, focuspunch=true, freezeshock=true, geomancy=true, holdhands=true, iceburn=true, magicaltorque=true, mefirst=true, metronome=true, mimic=true, mirrormove=true, naturepower=true, noxioustorque=true, phantomforce=true, razorwind=true, shadowforce=true, shelltrap=true, sketch=true, skullbash=true, skyattack=true, skydrop=true, sleeptalk=true, solarbeam=true, solarblade=true, struggle=true, uproar=true, wickedtorque=true}
-- Move 214. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "sleep-talk", name = "Sleep Talk", type = Type.Normal,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"calls_move"},
  usable_while_asleep = true,
  script = function(ctx)
    if not ctx.user.asleep then ctx:fail(); return end
    local choices = {}
    for _, id in ipairs(ctx.user.moves) do
      local info = ctx:move_info(id)
      if info and not info.flags.charge and not forbidden[id:gsub("[^%w]", ""):lower()] then choices[#choices+1]=id end
    end
    if #choices == 0 then ctx:fail(); return end
    ctx:call_move(choices[ctx:random_int(1, #choices)])
  end,
}
