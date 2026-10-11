-- Move 736: Glitzy Glow. Identity and parameters: pokemon_data/assets/moves/736.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
if ctx:damage(80).hit then
  ctx:start_effect(ctx.user.team, "light_screen", {turns = 5, rules = {{kind = Rule.DamageTaken, category = Category.Special, factor = 0.5, multi_battle_factor = 2732 / 4096, not_on_crit = true}}})
end
end}

move.id = "glitzy-glow"
move.name = "Glitzy Glow"
move.type = Type.Psychic
move.category = Category.Special
move.pp = 15
move.accuracy = 0.95
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
