-- Move 559: Fusion Bolt. Identity and parameters: pokemon_data/assets/moves/559.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
  script = function(ctx)
    ctx:damage(ctx.field.marks.fusion_flare and 200 or 100)
    ctx:mark(ctx.field, "fusion_bolt", 1, 1)
  end,
}

move.id = "fusion-bolt"
move.name = "Fusion Bolt"
move.type = Type.Electric
move.category = Category.Physical
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
