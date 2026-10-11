-- Move 497: Echoed Voice. Identity and parameters: pokemon_data/assets/moves/497.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.


local move = {
  script = function(ctx)
    -- All users share one counter, which increases at most once each battle turn.
    local echoes = ctx.field.marks.echoed_voice or 0
    if ctx.field.marks.echoed_voice_turn ~= ctx.battle_turn then echoes = math.min(echoes + 1, 5) end
    ctx:mark(ctx.field, "echoed_voice", echoes, 2)
    ctx:mark(ctx.field, "echoed_voice_turn", ctx.battle_turn, 2)
    ctx:damage(40 * echoes)
  end,
}

move.id = "echoed-voice"
move.name = "Echoed Voice"
move.type = Type.Normal
move.category = Category.Special
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"sound"}
move.base_power = 40
return move
