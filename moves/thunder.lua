-- Move 87: Thunder. Identity and parameters: pokemon_data/assets/moves/87.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {accuracy = 0.7, hits_hidden = {Hidden.Air},
    script = function(ctx)
      local hit = ctx:damage(110, {never_miss = ctx.weather == Weather.Rain, accuracy = ctx.weather == Weather.Sun and 0.5 or nil})
      if hit.hit then ctx:apply_status(ctx.target, Status.Paralyzed, {chance = 0.3}) end
    end}

move.id = "thunder"
move.name = "Thunder"
move.type = Type.Electric
move.category = Category.Special
move.pp = 10
move.accuracy = 0.7
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
