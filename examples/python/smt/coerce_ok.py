def main():
    a: int = 4
    b: bool = True
    __VERIFIER_assert(a+b == 5)
    a: int = 4
    b: bool = False
    __VERIFIER_assert(a+b == 4)
    b = True
    __VERIFIER_assert(a//b == 4)
    b = True
    c: bool = False
    __VERIFIER_assert(c-b == -1)
    __VERIFIER_assert(True == 1)
    __VERIFIER_assert(False == 0)
    __VERIFIER_assert(0 < (4 > 1))
    __VERIFIER_assert(True | 4 == 5)
    __VERIFIER_assert(True | True == True)
    __VERIFIER_assert(True ^ 5 == 4)
    __VERIFIER_assert(-True == -1)
    __VERIFIER_assert(-False == 0)