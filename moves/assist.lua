-- Gen 8 calling restrictions from Pokemon Showdown 942cda71.
local forbidden = {assist=true, banefulbunker=true, beakblast=true, belch=true, bestow=true, blazingtorque=true, bounce=true, burningbulwark=true, celebrate=true, chatter=true, circlethrow=true, combattorque=true, copycat=true, counter=true, covet=true, destinybond=true, detect=true, dig=true, dive=true, dragontail=true, endure=true, feint=true, fly=true, focuspunch=true, followme=true, helpinghand=true, holdhands=true, kingsshield=true, magicaltorque=true, matblock=true, mefirst=true, metronome=true, mimic=true, mirrorcoat=true, mirrormove=true, naturepower=true, noxioustorque=true, phantomforce=true, protect=true, ragepowder=true, roar=true, shadowforce=true, shelltrap=true, sketch=true, skydrop=true, sleeptalk=true, snatch=true, spikyshield=true, spotlight=true, struggle=true, switcheroo=true, terastarstorm=true, thief=true, transform=true, trick=true, whirlwind=true, wickedtorque=true}
-- Move 274. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "assist", name = "Assist", type = Type.Normal,
  category = Category.Status, pp = 20, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"calls_move"},
  script = function(ctx)
    local choices = {}
    for _, pokemon in ipairs(ctx.user.team.roster) do
      if pokemon.index ~= ctx.user.index then
        for _, id in ipairs(pokemon.moves) do
          if not forbidden[id:gsub("[^%w]", ""):lower()] and ctx:move_info(id) then choices[#choices+1]=id end
        end
      end
    end
    if #choices == 0 then ctx:fail(); return end
    ctx:call_move(choices[ctx:random_int(1, #choices)])
  end,
}
