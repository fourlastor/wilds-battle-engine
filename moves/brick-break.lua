-- Move 280: Brick Break. Identity and parameters: pokemon_data/assets/moves/280.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    for _, screen in ipairs({"reflect", "light_screen", "aurora_veil"}) do ctx:end_effect(ctx.target.team, screen) end
    ctx:damage(75)
  end,
}

move.id = "brick-break"
move.name = "Brick Break"
move.type = Type.Fighting
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
move.base_power = 75
return move
