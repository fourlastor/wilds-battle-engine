-- Move 479: Smack Down. Identity and parameters: pokemon_data/assets/moves/479.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
  script = function(ctx)
    if not ctx:damage(50).hit then return end
    ctx:unhide(ctx.target)
    ctx:start_effect(ctx.target, "smacked_down", {rules = {{kind = Rule.Grounded}}})
  end,
}

move.id = "smack-down"
move.name = "Smack Down"
move.type = Type.Rock
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
move.hits_hidden = {Hidden.Air}
return move
