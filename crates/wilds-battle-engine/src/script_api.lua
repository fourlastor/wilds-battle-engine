-- A yielded operation is applied by Rust before its result is returned to Lua.
return function(api, user, target, statuses)
  local ctx = {user=user, target=target, Status=statuses}

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

  return ctx
end
