def __VERIFIER_assert(a):
    pass

def __VERIFIER_assume(a):
    pass

def main(a: int, b: int, c: int, d: int):
    """
    pre: True
    """
    unreachable: bool = True
    if(b > 0):
        if(a//b == d):
            for i in range(10):
                a = a + d
            if a*d > 0:
                unreachable = False
                assert False
    __VERIFIER_assert(unreachable)

