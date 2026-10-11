-- Move 518: Water Pledge. Identity and parameters: pokemon_data/assets/moves/518.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: The example implements side-effect combinations only. Combined 150-power attacks, forced STAB and cross-move queue promotion need engine support.

local move = {
    script = function(ctx)
      local hit = ctx:damage(80)
      if hit.hit and ctx.user.team.marks.grass_pledge then
        ctx:message("A swamp enveloped the opposing team!")
        ctx:start_effect(ctx.target.team, "swamp", {turns = 4, rules = {{kind = Rule.StatMultiplier, stat = Stat.Speed, factor = 0.5}}})
      end
      ctx:mark(ctx.user.team, "water_pledge", 1, 1)
    end}

move.id = "water-pledge"
move.name = "Water Pledge"
move.type = Type.Water
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"incomplete"}
return move
