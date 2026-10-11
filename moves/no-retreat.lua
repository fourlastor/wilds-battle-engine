-- Move 748: No Retreat. Identity and parameters: pokemon_data/assets/moves/748.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Trapping is recorded as a Trapped rule for the host; the engine has no switching or bench.

local move = {script = function(ctx)
if ctx.user.marks.no_retreat then return ctx:fail() end
local changed = false
for _, stat in ipairs({Stat.Attack, Stat.Defense, Stat.SpAttack, Stat.SpDefense, Stat.Speed}) do
  changed = ctx:change_stat(ctx.user, stat, 1) ~= 0 or changed
end
if not changed then return ctx:fail() end
if not ctx.user.trapped then
  local rules = ctx.user:has_type(Type.Ghost) and {} or {{kind = Rule.Trapped}}
  ctx:start_effect(ctx.user, "no_retreat", {rules = rules})
end
end}

move.id = "no-retreat"
move.name = "No Retreat"
move.type = Type.Fighting
move.category = Category.Status
move.pp = 5
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch", "incomplete"}
return move
