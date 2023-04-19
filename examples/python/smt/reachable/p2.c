int main(int a, int b, int c, int d){
    int unreachable = 0;
    a = a ^ b;
    if(a<(c / 2)){
        for(int i=0; i <30; i++){
            d = c + b;
            for(int j=20; j <40; j++){
                d = d + (b/2);
            }
        }
        if(d > a){
            unreachable = 1;
        }
    }
    __VERIFIER_assert(unreachable==0);
}