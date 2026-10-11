-- Move 542: Hurricane. Identity and parameters: pokemon_data/assets/moves/542.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {accuracy = 0.7, hits_hidden = {Hidden.Air},
    script = function(ctx)
      local hit = ctx:damage(110, {never_miss = ctx.weather == Weather.Rain, accuracy = ctx.weather == Weather.Sun and 0.5 or nil})
      if hit.hit then ctx:confuse(ctx.target, {chance = 0.3}) end
    end}

move.id = "hurricane"
move.name = "Hurricane"
move.type = Type.Flying
move.category = Category.Special
move.pp = 10
move.accuracy = 0.7
move.priority = 0
move.target = Target.Selected
move.flags = {"wind"}
move.base_power = 110
return move
