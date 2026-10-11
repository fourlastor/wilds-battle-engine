-- Move 807: Burning Jealousy. Identity and parameters: pokemon_data/assets/moves/807.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
ctx:damage(70)
for _, foe in ipairs(ctx.foes) do
  if foe.stats_raised_this_turn and ctx:reached(foe) then
    ctx:apply_status(foe, Status.Burned)
  end
end
end}

move.id = "burning-jealousy"
move.name = "Burning Jealousy"
move.type = Type.Fire
move.category = Category.Special
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.AllOpponents
move.flags = {}
move.base_power = 70
return move
