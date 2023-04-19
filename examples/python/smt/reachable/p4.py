def __VERIFIER_assert(a):
    pass

def __VERIFIER_assume(a):
    pass

def main(a: int, b: int, c: int, d: int):
    """
    pre: True
    """
    unreachable: bool = True
    if(a ^ b  == 1):
        if(c | d == 0):
            for i in range(10):
                b = b + c
                a = a - d
            if(b == d) & (a == d):
                unreachable = False
                assert False
    __VERIFIER_assert(unreachable)

