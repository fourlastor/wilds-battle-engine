-- Gen 8 copying restrictions from Pokemon Showdown 942cda71.
local forbidden = {assist=true, banefulbunker=true, beakblast=true, behemothbash=true, behemothblade=true, belch=true, bestow=true, blazingtorque=true, burningbulwark=true, celebrate=true, chatter=true, circlethrow=true, combattorque=true, copycat=true, counter=true, covet=true, destinybond=true, detect=true, dragontail=true, dynamaxcannon=true, endure=true, feint=true, focuspunch=true, followme=true, helpinghand=true, holdhands=true, kingsshield=true, magicaltorque=true, matblock=true, mefirst=true, metronome=true, mimic=true, mirrorcoat=true, mirrormove=true, naturepower=true, noxioustorque=true, protect=true, ragepowder=true, roar=true, shelltrap=true, sketch=true, sleeptalk=true, snatch=true, spikyshield=true, spotlight=true, struggle=true, switcheroo=true, terastarstorm=true, thief=true, transform=true, trick=true, whirlwind=true, wickedtorque=true}
-- Move 383. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "copycat", name = "Copycat", type = Type.Normal,
  category = Category.Status, pp = 20, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"calls_move"},
  script = function(ctx)
    local move = ctx.last_move
    if not move or forbidden[move:gsub("[^%w]", ""):lower()] then ctx:fail(); return end
    ctx:call_move(move)
  end,
}
