-- Move 228: Pursuit. Identity and parameters: pokemon_data/assets/moves/228.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  ctx:damage(ctx.target.switching and 80 or 40, {never_miss = ctx.target.switching})
end}

move.id = "pursuit"
move.name = "Pursuit"
move.type = Type.Dark
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "intercepts_switch"}
move.base_power = 40
return move
