-- Move 290: Secret Power. Identity and parameters: pokemon_data/assets/moves/290.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
    script = function(ctx)
      local hit = ctx:damage(70)
      if not hit.hit or not ctx:effect_chance(0.3) then return end
      local terrain = ctx.field.effects
      if terrain.electric_terrain then return ctx:apply_status(ctx.target, Status.Paralyzed) end
      if terrain.grassy_terrain then return ctx:apply_status(ctx.target, Status.Asleep) end
      if terrain.misty_terrain then return ctx:change_stat(ctx.target, Stat.SpAttack, -1) end
      if terrain.psychic_terrain then return ctx:change_stat(ctx.target, Stat.Speed, -1) end
      local place = ctx.environment
      if place == "cave" or place == "mountain" then ctx:flinch(ctx.target)
      elseif place == "water" then ctx:change_stat(ctx.target, Stat.Attack, -1)
      elseif place == "desert" or place == "road" then ctx:change_stat(ctx.target, Stat.Accuracy, -1)
      elseif place == "grass" then ctx:apply_status(ctx.target, Status.Asleep)
      elseif place == "snow" then ctx:apply_status(ctx.target, Status.Frozen)
      else ctx:apply_status(ctx.target, Status.Paralyzed) end
    end}

move.id = "secret-power"
move.name = "Secret Power"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
move.base_power = 70
return move
