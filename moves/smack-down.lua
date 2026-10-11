-- Move 479: Smack Down. Identity and parameters: pokemon_data/assets/moves/479.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
  script = function(ctx)
    local hit = ctx:damage(50)
    if not hit.hit or hit.substitute then return end
    ctx:end_effect(ctx.target, "magnet_rise")
    ctx:end_effect(ctx.target, "telekinesis")
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
