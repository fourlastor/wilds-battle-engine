-- A yielded operation is applied by Rust before its result is returned to Lua.
return function(api, user, target, statuses, weather, turn, total_turns, target_policy, stats, types)
  local ctx = {user=user, target=target, Status=statuses, weather=weather,
    turn=turn, total_turns=total_turns, TargetPolicy=target_policy, Stat=stats, Type=types}

  function ctx:damage(power, options)
    return coroutine.yield(api.damage(power, options))
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

  return ctx
end
