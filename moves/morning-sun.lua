return {
  id="morning_sun", name="Morning Sun", type=Type.Normal, category=Category.Status,
  pp=5, target=Target.User, accuracy=Accuracy.Always,
  script=function(ctx)
    local fraction = 0.5
    if ctx.weather == Weather.Sun then fraction = 2/3 end
    if ctx.weather == Weather.Sandstorm then fraction = 0.25 end
    ctx:heal_self(fraction)
  end
}
