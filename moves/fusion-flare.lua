-- Move 558: Fusion Flare. Identity and parameters: pokemon_data/assets/moves/558.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
  script = function(ctx)
    ctx:damage(ctx.field.marks.fusion_bolt and 200 or 100)
    ctx:mark(ctx.field, "fusion_flare", 1, 1)
  end,
}

move.id = "fusion-flare"
move.name = "Fusion Flare"
move.type = Type.Fire
move.category = Category.Special
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
move.usable_while_frozen = true
return move
