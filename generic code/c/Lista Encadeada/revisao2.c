/*Um bom exemplo de aplicação de pilha é o funcionamento das calculadoras da HP (HewlettPackard). Elas trabalham com expressões pós-fixadas, então para avaliarmos uma expressão como
(1-2)*(4+5) podemos digitar 1 2 – 4 5 + *. O funcionamento dessas calculadoras é muito simples.
Cada operando é empilhado numa pilha de valores.
Quando se encontra um operador, desempilha-se o número apropriado de operandos, realiza-se a
operação devida e empilha-se o resultado.
Deste modo, na expressão acima, são empilhados os valores 1 e 2. Quando aparece o operador -, 1
e 2 são desempilhados e o resultado da operação, no caso -1 (= 1 - 2), é colocado no topo da pilha.
A seguir, 4 e 5 são empilhados.
O operador seguinte, +, desempilha o 4 e o 5 e empilha o resultado da soma, 9.
Nesta hora, estão na pilha os dois resultados parciais, -1 na base e 9 no topo.
O operador *, então, desempilha os dois e coloca -9 (= -1 * 9) no topo da pilha.*/
#include <stdio.h>
#include <stdlib.h>
#include <conio.h>

typedef struct no{
	int x;
	struct no* prox;
}lista;

int somar(lista** inicio){
    int valor = 0;
    lista* pos;
    if(*inicio == NULL){
        printf("Lista Vazia\n");
    }
    else{
        pos = *inicio;
        while(pos != NULL){
            valor += pos->x;
            pos = pos->prox;
        }
        return valor;
    }
    return NULL;
}

int subtrair(lista** inicio){
    int valor = 0;
    lista* pos;
    if(*inicio == NULL){
        printf("Lista Vazia\n");
    }
    else{
        pos = *inicio;
        while(pos != NULL){
            valor -= pos->x;
            pos = pos->prox;
        }
        return valor;
    }
    return NULL;
}

int multiplicar(lista** inicio){
    int valor = 1;
    lista* pos;
    if(*inicio == NULL){
        printf("Lista Vazia\n");
    }
    else{
        pos = *inicio;
        while(pos != NULL){
            valor *= pos->x;
            pos = pos->prox;
        }
        return valor;
    }
    return NULL;
}

int dividir(lista** inicio){
    int valor = 1;
    lista* pos;
    if(*inicio == NULL){
        printf("Lista Vazia\n");
    }
    else{
        pos = *inicio;
        while(pos != NULL){
            valor /= pos->x;
            pos = pos->prox;
        }
        return valor;
    }
    return NULL;
}

void pedir_valor(int* x, lista** inicio){
    char digito[10];
	printf("Digite um valor: ");
    scanf("%s", digito);
    if(digito[0] >= '0'){
        *x = atoi(digito);
    }
    else{
        switch(digito[0]){
            case '+':
                *x = somar(inicio);
                while(*inicio != NULL){
                    excluir_inicio(inicio);
                }
                break;
            case '-':
                *x = subtrair(inicio);
                while(*inicio != NULL){
                    excluir_inicio(inicio);
                }
                break;
            case '*':
                *x = multiplicar(inicio);
                while(*inicio != NULL){
                    excluir_inicio(inicio);
                }
                break;
            case '/':
                *x = dividir(inicio);
                while(*inicio != NULL){
                    excluir_inicio(inicio);
                }
                break;
        }
    }
    
}

void imprimir_lista (lista* inicio){
	lista *pos;
	
	pos = inicio;
	
	printf("\n");
	
	if(inicio == NULL){
		printf("\nLista Vazia");
	}
	else{
		while (pos != NULL){
			printf("\n%d | [%d] -> %d", pos, pos->x, pos->prox);
			pos = pos->prox;
		}
	}
}

void inserir_inicio (lista** inicio){
	int x;
	lista *novo;
	
	novo = (lista*) malloc (sizeof(lista));
	
	if(novo == NULL){
		printf("\nERRO");
	}
	else{
		pedir_valor(&x, inicio);
		
		novo->x = x;
		novo->prox = *inicio;
		*inicio = novo;
	}
}

void excluir_inicio (lista** inicio){
	lista* pos;
	
	if(*inicio == NULL){
		printf("\nLista Vazia");		
	}
	else{
		pos = *inicio;
		*inicio = pos->prox;
		
		free(pos);
	}
}

void menu (int* op){
	do{
		printf("\n\n\tMENU\n");
		printf("\n----------");
		printf("\n1 - Inserir na Pilha\n2 - Excluir da Pilha\n3 - Sair");
		printf("\n----------\n");
		*op = _getche() - '0';
	}while (*op < 1 || *op > 3);
}

int main(){
    lista *inicio;
    int op;

    inicio = NULL;

    do{
        system("cls");
        imprimir_lista(inicio);
        menu(&op);
        switch(op){
            case 1:
                inserir_inicio(&inicio);
                break;
            case 2:
                excluir_inicio(&inicio);
                break;
        }
    }while(op != 3);
}