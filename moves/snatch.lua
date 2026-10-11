-- Gen 8 beneficial move eligibility; Pokemon Showdown 942cda71.
local eligible = {["acidarmor"]=true, ["agility"]=true, ["amnesia"]=true, ["aquaring"]=true, ["aromatherapy"]=true, ["auroraveil"]=true, ["autotomize"]=true, ["barrier"]=true, ["bellydrum"]=true, ["bulkup"]=true, ["calmmind"]=true, ["camouflage"]=true, ["charge"]=true, ["clangoroussoul"]=true, ["coil"]=true, ["conversion"]=true, ["cosmicpower"]=true, ["cottonguard"]=true, ["defendorder"]=true, ["defensecurl"]=true, ["doubleteam"]=true, ["dragondance"]=true, ["focusenergy"]=true, ["gearup"]=true, ["growth"]=true, ["harden"]=true, ["healbell"]=true, ["healingwish"]=true, ["healorder"]=true, ["honeclaws"]=true, ["howl"]=true, ["imprison"]=true, ["ingrain"]=true, ["irondefense"]=true, ["laserfocus"]=true, ["lifedew"]=true, ["lightscreen"]=true, ["luckychant"]=true, ["lunardance"]=true, ["magneticflux"]=true, ["magnetrise"]=true, ["matblock"]=true, ["meditate"]=true, ["milkdrink"]=true, ["minimize"]=true, ["mist"]=true, ["moonlight"]=true, ["morningsun"]=true, ["nastyplot"]=true, ["noretreat"]=true, ["powertrick"]=true, ["quickguard"]=true, ["quiverdance"]=true, ["recover"]=true, ["recycle"]=true, ["reflect"]=true, ["refresh"]=true, ["rest"]=true, ["rockpolish"]=true, ["roost"]=true, ["safeguard"]=true, ["sharpen"]=true, ["shellsmash"]=true, ["shiftgear"]=true, ["shoreup"]=true, ["slackoff"]=true, ["softboiled"]=true, ["stockpile"]=true, ["stuffcheeks"]=true, ["substitute"]=true, ["swallow"]=true, ["swordsdance"]=true, ["synthesis"]=true, ["tailglow"]=true, ["tailwind"]=true, ["wideguard"]=true, ["wish"]=true, ["withdraw"]=true, ["workup"]=true}
-- Move 289. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "snatch", name = "Snatch", type = Type.Dark,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 4, target = Target.User,
  flags = {"bypass_substitute"},
  script = function(ctx)
    local moves = {}
    for _, id in ipairs(ctx:move_ids()) do
      if eligible[id:gsub("[^%w]", ""):lower()] then moves[#moves+1]=id end
    end
    ctx:start_effect(ctx.user, "snatch", {turns=1, rules={{kind=Rule.Snatch, moves=moves}}})
  end,
}
