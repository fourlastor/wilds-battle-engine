-- Move 706: Psychic Fangs. Identity and parameters: pokemon_data/assets/moves/706.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    for _, screen in ipairs({"reflect", "light_screen", "aurora_veil"}) do ctx:end_effect(ctx.target.team, screen) end
    ctx:damage(85)
  end,
}

move.id = "psychic-fangs"
move.name = "Psychic Fangs"
move.type = Type.Psychic
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "bite"}
return move
