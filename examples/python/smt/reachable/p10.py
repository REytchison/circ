def __VERIFIER_assert(a):
    pass

def __VERIFIER_assume(a):
    pass

def main(a: int, b: int, c: int, d: int):
    """
    pre: True
    """
    unreachable: bool = True

    for i in range(1000):
        a = a + c
        d = b - c
    
    if(a != d):
        unreachable = False
        assert False
    
    __VERIFIER_assert(unreachable)

