# some basic stuff like functions
#f = x => x + 1
#:> f(1)
even = n => {:> n % 2 == 0}
halve = n => {even(n); :> n / 2}

:> halve(12)

#f = x => {:> x + 1}
#x = 2
#{:> x > 1}
#f = x => {
#    :> x + 1
#}
#
## assertions
#f = (a, b) => {a > b; :> a - b}
#f = (a, b) => {
#    a > b
#    :> a - b
#}
#
## conditions
#positive = x => {
#    x < 0 -> :> -x
#    :> x
#}
#
## mapping looping
#a = (0..5) :: x => x**2
#b = a :: x => x - 1
#
## bool functions vs asserting
#f = x => {:> x > 0} # returns bool
#f = x => x > 0  # shorthand for the above func
#
#f = x => {x > 0} # assertion shorthand for
#f = x => {x > 0; :> true}
#
## working with result
#len = l => {
#    length = 0
#    l :: a => length += 1
#    :> length
#}
#
#len = l => {
#    l :: a => result += 1
#}
