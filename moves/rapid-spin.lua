-- Move 229: Rapid Spin. Identity and parameters: pokemon_data/assets/moves/229.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
if not ctx:damage(50).hit then return end
ctx:change_stat(ctx.user, Stat.Speed, 1)
ctx:free(ctx.user)
ctx:end_effect(ctx.user, "leech_seed")
for _, hazard in ipairs({"spikes", "toxic_spikes", "stealth_rock", "sticky_web"}) do
  ctx:end_effect(ctx.user.team, hazard)
end
end}

move.id = "rapid-spin"
move.name = "Rapid Spin"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 40
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "sheer_force"}
move.base_power = 50
return move
