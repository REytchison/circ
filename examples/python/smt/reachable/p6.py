def __VERIFIER_assert(a):
    pass

def __VERIFIER_assume(a):
    pass

def main(a: int, b: int, c: int, d: int):
    """
    pre: True
    """
    unreachable: bool = True
    if(a < 0):
        a = a ^ a
    if(b < 0):
        b = b ^ b
    if(c < 0):
        c = c ^ c
    if(d < 0):
        d = d ^ d
    if a + b < 0:
        unreachable = False
        assert False
    __VERIFIER_assert(unreachable)

