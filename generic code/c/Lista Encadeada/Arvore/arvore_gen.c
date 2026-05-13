#include <stdio.h>
#include <stdlib.h>

struct NO{
    int valor;
    struct NO* filho;
    struct NO* irmão;
}arvore;

void inserir(){
    
}

void menu(int* op){
    do{
        printf("\n\tMENU\n");
        printf("-----------\n");
        printf("\n (1) INSERIR \n");
        printf("\n (2) BUSCAR \n");
        printf("\n (3) REMOVER \n");
        printf("\n (4) LISTAR \n");
        printf("-----------\n")
    }while(*op < 0 || *op > 4);
}

int main(){
    int op;

    while(op != 0){
        switch(op){
            case 1:

                break;
            case 2:
                break;
            case 3:
                break;
            case 4:
                break;
        }
    }
}