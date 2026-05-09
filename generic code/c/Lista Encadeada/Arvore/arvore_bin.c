#include <stdio.h>
#include <stdlib.h>

typedef struct no{
    int valor;
    struct no *esq;
    struct no *dir;
}arvore;

void inserir(arvore** raiz, int valor){
    if(*raiz == NULL){
        arvore *novo = (arvore*)malloc(sizeof(arvore));
        novo->valor = valor;
        novo->esq = NULL;
        novo->dir = NULL;
        *raiz = novo;
    }else{
        if(valor < (*raiz)->valor){
            inserir(&(*raiz)->esq, valor);
        }else{
            inserir(&(*raiz)->dir, valor);
        }
    }
}

void buscar(arvore* raiz, int valor){
    if(raiz == NULL){
        printf("\n O valor nao foi encontrado\n");
    }else{
        if(valor == raiz->valor){
            printf("\n O valor foi encontrado\n");
        }else{
            if(valor < raiz->valor){
                buscar(raiz->esq, valor);
            }else{
                buscar(raiz->dir, valor);
            }
        }
    }
}

void remover(arvore* raiz, int valor){
    if(raiz == NULL){
        printf("\n O valor nao foi encontrado\n");
    }else{
        if(valor == raiz->valor){
            if(raiz->esq == NULL && raiz->dir == NULL){
                free(raiz);
            }else{
                if(raiz->esq == NULL){
                    arvore *temp = raiz->dir;
                    free(raiz);
                    raiz = temp;
                }else{
                    arvore *temp = raiz->esq;
                    free(raiz);
                    raiz = temp;
                }
            }
        }else{
            if(valor < raiz->valor){
                remover(raiz->esq, valor);
            }else{
                remover(raiz->dir, valor);
            }
        }
    }
}

void imprimir_preOrder(arvore *raiz, int tab){
    if(raiz != NULL){
        for(int i = 0; i < tab; i++){
            printf("\t");
        }
        printf("%d <- [%d] *%d -> %d \n", raiz->esq, raiz->valor, *raiz, raiz->dir);
        if(raiz->esq != NULL){
            imprimir_preOrder(raiz->esq, tab + 1);
        }
        if(raiz->dir != NULL){
            imprimir_preOrder(raiz->dir, tab + 1);
        }
    }
    else{
        printf("\n A arvore esta vazia\n");
    }
}

void imprimir_posOrder(arvore *raiz, int tab){
    if(raiz != NULL){
        if(raiz->esq != NULL){
            imprimir_posOrder(raiz->esq, tab + 1);
        }
        if(raiz->dir != NULL){
            imprimir_posOrder(raiz->dir, tab + 1);
        }
        for(int i = 0; i < tab; i++){
            printf("\t");
        }
        printf("%d <- [%d] *%d -> %d \n", raiz->esq, raiz->valor, *raiz, raiz->dir);
    }
    else{
        printf("\n A arvore esta vazia\n");
    }
}

void imprimir_inOrder(arvore *raiz, int tab){
    if(raiz != NULL){
        if(raiz->esq != NULL){
            imprimir_inOrder(raiz->esq, tab + 1);
        }
        for(int i = 0; i < tab; i++){
            printf("\t");
        }
        printf("%d <- [%d] *%d -> %d \n", raiz->esq, raiz->valor, *raiz, raiz->dir);
        if(raiz->dir != NULL){
            imprimir_inOrder(raiz->dir, tab + 1);
        }
    }
    else{
        printf("\n A arvore esta vazia\n");
    }
}

int menu(){
    int op;
    do{
        printf("\n --- MENU ---\n");
        printf("\n (1) INSERIR \n");
        printf("\n (2) BUSCAR \n");
        printf("\n (3) REMOVER \n");
        printf("\n (4) LISTAR \n");
        printf("\n (0) SAIR \n");
        scanf("%d", &op);
    }while(op < 0 || op > 4);
    return op;
}

int main(){
    arvore *raiz = NULL;
    int op = -1, imprimir = 0;
    
    while(op != 0){
        op = menu();

        switch(op){
            case 1:
                printf("Digite o valor: ");
                int valor;
                scanf("%d", &valor);
                inserir(&raiz, valor);
                break;
            case 2:
                printf("Digite o valor: ");
                scanf("%d", &valor);
                buscar(raiz, valor);
                break;
            case 3:
                printf("Digite o valor: ");
                scanf("%d", &valor);
                remover(&raiz, valor);
                break;
            case 4:
                do{
                    printf("\n (1) PRE-ORDER \n");
                    printf("\n (2) POS-ORDER \n");
                    printf("\n (3) IN-ORDER \n");
                    printf("\n (4) SAIR \n");
                    scanf("%d", &imprimir);
                }while(imprimir < 1 || imprimir > 4);
                switch(imprimir){
                    case 1:
                        imprimir_preOrder(raiz, 0);
                        break;
                    case 2:
                        imprimir_posOrder(raiz, 0);
                        break;
                    case 3:
                        imprimir_inOrder(raiz, 0);
                        break;
                }
                break;
        }
    }
    return 0;
}