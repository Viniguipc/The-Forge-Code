#include <stdio.h>
#include <stdlib.h>

typedef struct no{
    int valor;
    struct no *esq;
    struct no *dir;
}arvore;

void inserir(arvore* raiz){
    
}

void menu(int *op){
    do{
        printf("\n --- MENU ---\n");
        printf("\n (1) INSERIR  \n");
        printf("\n (2) BUSCAR   \n");
        printf("\n (3) REMOVER  \n");
        printf("\n (4) LISTAR   \n");
        printf("\n (0) SAIR     \n");
        scanf("%d", op);
    }while(op < 0 || op > 4);
}
main(){
    arvore *raiz = NULL;
    int op = 0;
    
    while(op != 0){
        menu(&op);
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