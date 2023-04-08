def main(a: int):
    __VERIFIER_assume(a != 4)
    if(a == 4):
        a = a + 1
    else:
        a = 4
    __VERIFIER_assert(a == 4)