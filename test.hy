f = x => x + 1
g = a => a + f(a)

test = 4
:> f( g(test) )