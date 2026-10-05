-- A yielded operation is applied by Rust before its result is returned to Lua.
-- `fields` holds what the script can read (ctx.user, ctx.target, ctx.weather, ...). Rust refreshes
-- those tables after every operation, so they always show the battle as it is now.
-- `roster` lists the table of every Pokémon in the battle, fainted or not.
return function(api, fields, roster)
  local ctx = fields

  -- Helpers on every Pokémon table.
  local Pokemon = {}
  Pokemon.__index = Pokemon
  function Pokemon:has_type(kind)
    for _, own in ipairs(self.types) do
      if own == kind then return true end
    end
    return false
  end
  function Pokemon:stage(stat)
    return self.stages[stat] or 0
  end
  -- How many stages its five battle stats have been raised in total; lowered stats do not count.
  function Pokemon:raised_stages()
    local total = 0
    for _, stat in ipairs({Stat.Attack, Stat.Defense, Stat.SpAttack, Stat.SpDefense, Stat.Speed}) do
      total = total + math.max(self:stage(stat), 0)
    end
    return total
  end
  -- Whether it has used every one of its moves except `move_id` since it entered the battle.
  function Pokemon:used_all_other_moves(move_id)
    local others = 0
    for _, id in ipairs(self.moves) do
      if id ~= move_id then
        if not self.used_moves[id] then return false end
        others = others + 1
      end
    end
    return others > 0
  end
  for _, pokemon in ipairs(roster) do setmetatable(pokemon, Pokemon) end

  local function op(name, ...)
    return coroutine.yield(api.op(name, ...))
  end

  -- Whether this use of the move has got through to each Pokémon it tried to hit.
  local reached = {}
  local function note(result)
    for _, position in ipairs(result.unreached) do reached[roster[position]] = false end
    for _, position in ipairs(result.reached) do reached[roster[position]] = true end
    return result
  end

  function ctx:damage(power, options)
    return note(coroutine.yield(api.damage(power, options)))
  end

  function ctx:faint_user()
    return coroutine.yield(api.faint_user())
  end

  function ctx:recharge()
    return coroutine.yield(api.recharge())
  end

  function ctx:fail()
    return coroutine.yield(api.fail())
  end

  function ctx:force_move(total, policy)
    return coroutine.yield(api.force_move(total, policy))
  end

  function ctx:break_sequence()
    return coroutine.yield(api.break_sequence())
  end

  function ctx:random_int(min, max)
    return coroutine.yield(api.random_int(min, max))
  end

  function ctx:message(message)
    return coroutine.yield(api.message(message))
  end

  function ctx:announce()
    return coroutine.yield(api.announce())
  end

  function ctx:confuse_self()
    return coroutine.yield(api.confuse_self())
  end

  function ctx:recoil_max_hp(fraction)
    return coroutine.yield(api.recoil_max_hp(fraction))
  end

  function ctx:change_self_stat(stat, stages)
    return coroutine.yield(api.change_self_stat(stat, stages))
  end

  function ctx:boost_next_move(move_type, multiplier)
    return coroutine.yield(api.boost_next_move(move_type, multiplier))
  end

  function ctx:watch_hits_until_next_action()
    return coroutine.yield(api.watch_hits_until_next_action())
  end

  function ctx:heal_self(fraction)
    return coroutine.yield(api.heal_self(fraction))
  end

  function ctx:flinch_target(chance)
    return coroutine.yield(api.flinch_target(chance))
  end

  -- Hitting and damage
  function ctx:try_hit(who, options)
    local hit = op("try_hit", who, options)
    reached[who or ctx.target] = hit
    return hit
  end
  -- Whether the move has got through to `who` (the target if left out). The first thing a move
  -- does to a Pokémon decides it: a hit that landed or missed, or else the usual checks made now.
  function ctx:reached(who)
    who = who or ctx.target
    if who == ctx.user then return true end
    if reached[who] == nil then return ctx:try_hit(who) end
    return reached[who]
  end
  function ctx:chance(probability) return op("chance", probability) end
  function ctx:effect_chance(probability) return op("effect_chance", probability) end
  function ctx:multi_hit(power, min, max, options) return note(op("multi_hit", power, min, max, options)) end
  function ctx:direct_damage(who, amount, options) return note(op("direct_damage", who, amount, options)) end
  function ctx:hurt(who, amount) return op("hurt", who, amount) end
  function ctx:heal(who, amount) return op("heal", who, amount) end

  -- Conditions of a Pokémon
  function ctx:apply_status(who, status, options) return op("apply_status", who, status, options) end
  function ctx:cure_status(who, status) return op("cure_status", who, status) end
  function ctx:confuse(who, options) return op("confuse", who, options) end
  function ctx:flinch(who, options) return op("flinch", who, options) end
  function ctx:change_stat(who, stat, stages, options) return op("change_stat", who, stat, stages, options) end
  function ctx:reset_stats(who) return op("reset_stats", who) end
  function ctx:bind(who, options) return op("bind", who, options) end
  function ctx:free(who) return op("free", who) end
  function ctx:protect(who) return op("protect", who or ctx.user) end
  function ctx:break_protect(who) return op("break_protect", who or ctx.target) end
  function ctx:hide(kind) return op("hide", kind) end
  function ctx:unhide(who) return op("unhide", who) end
  function ctx:set_types(who, types) return op("set_types", who, types) end
  function ctx:set_weight(who, kilograms) return op("set_weight", who, kilograms) end
  function ctx:take_item(who) return op("take_item", who) end
  function ctx:give_item(who, item) return op("give_item", who, item) end
  function ctx:suppress_ability(who) return op("suppress_ability", who) end

  -- The field, marks and timed effects
  function ctx:set_weather(kind, turns) return op("set_weather", kind, turns) end
  function ctx:clear_weather() return op("clear_weather") end
  function ctx:mark(scope, name, value, turns) return op("mark", scope, name, value, turns) end
  function ctx:unmark(scope, name) return op("unmark", scope, name) end
  function ctx:start_effect(scope, name, options) return op("start_effect", scope, name, options) end
  function ctx:end_effect(scope, name) return op("end_effect", scope, name) end
  function ctx:end_group(scope, group) return op("end_group", scope, group) end

  -- The rest of the battle
  function ctx:add_payout(amount) return op("add_payout", amount) end
  function ctx:hurry() return op("hurry") end

  -- Shorthands built from the operations above.

  -- Moves every raised stat stage from one Pokémon to another (Spectral Thief).
  function ctx:steal_boosts(from, to)
    for _, stat in ipairs({Stat.Attack, Stat.Defense, Stat.SpAttack, Stat.SpDefense, Stat.Speed,
        Stat.Accuracy, Stat.Evasion}) do
      local stages = math.min(from:stage(stat), 6 - to:stage(stat))
      if stages > 0 then
        ctx:change_stat(from, stat, -stages)
        ctx:change_stat(to, stat, stages)
      end
    end
  end
  -- Gives `to` the item `from` holds, if `to` holds none. Returns the item, or nil.
  function ctx:steal_item(from, to)
    if not from.item or to.item then return nil end
    local item = ctx:take_item(from)
    ctx:give_item(to, item)
    return item
  end
  -- Takes one type away from a Pokémon for as long as it stays in the battle.
  function ctx:lose_type(who, kind)
    local kept = {}
    for _, own in ipairs(who.types) do
      if own ~= kind then kept[#kept + 1] = own end
    end
    if #kept == 0 then kept = {Type.None} end
    ctx:set_types(who, kept)
  end
  function ctx:opposite_genders(a, b)
    return a.gender ~= Gender.Genderless and b.gender ~= Gender.Genderless and a.gender ~= b.gender
  end
  function ctx:share_type(a, b)
    for _, kind in ipairs(a.types) do
      if kind ~= Type.None and b:has_type(kind) then return true end
    end
    return false
  end
  -- The type with a number from 0 to 15, in the order Hidden Power uses.
  local numbered = {Type.Fighting, Type.Flying, Type.Poison, Type.Ground, Type.Rock, Type.Bug,
    Type.Ghost, Type.Steel, Type.Fire, Type.Water, Type.Grass, Type.Electric, Type.Psychic, Type.Ice,
    Type.Dragon, Type.Dark}
  function ctx:type_number(number)
    return numbered[math.floor(number) % 16 + 1]
  end

  return ctx
end
