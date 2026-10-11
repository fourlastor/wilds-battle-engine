-- Move 737: Baddy Bad. Identity and parameters: pokemon_data/assets/moves/737.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
if ctx:damage(80).hit then
  ctx:start_effect(ctx.user.team, "reflect", {turns = 5, rules = {{kind = Rule.DamageTaken, category = Category.Physical, factor = 0.5, multi_battle_factor = 2732 / 4096, not_on_crit = true}}})
end
end}

move.id = "baddy-bad"
move.name = "Baddy Bad"
move.type = Type.Dark
move.category = Category.Special
move.pp = 15
move.accuracy = 0.95
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
