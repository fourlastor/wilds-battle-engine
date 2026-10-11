-- Move 477. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "telekinesis", name = "Telekinesis", type = Type.Psychic,
  category = Category.Status, pp = 15, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {"reflectable", "gravity"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    local who = ctx.target
    local species = (who.species or ""):gsub("[^%w]", ""):lower()
    local form = (who.form or ""):gsub("[^%w]", ""):lower()
    local rooted = {[50]=true, [51]=true, [769]=true, [770]=true}
    if who.grounded or rooted[who.species_id] or species == "diglett" or species == "dugtrio" or species == "sandygast" or species == "palossand" or species == "gengarmega" or ((who.species_id == 94 or species == "gengar") and (form == "mega" or form == "gengarmega")) then ctx:fail(); return end
    if not ctx:start_effect(who, "telekinesis", {turns = 3, rules = {{kind = Rule.Airborne}, {kind = Rule.IncomingAlwaysHit}}}) then ctx:fail() end
  end,
}
