#include <stdio.h>
#include <conio.h>
#include <stdlib.h>

typedef struct no{
	int x;
	struct no* prox;
}lista;

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

void menu (int* op){
	do{
		printf("\n\n\tMENU\n");
		printf("\n----------");
		printf("\n1 - Inserir na Pilha\n2 - Excluir da Pilha\n3 - Sair");
		printf("\n----------\n");
		*op = _getche() - '0';
	}while (*op < 1 || *op > 3);
}

void pedir_valor(int* x, int op){
	if(op == 1){
		printf("\nDigite o valor: ");
		scanf(" %d", x);
	}
	else{
		printf("\nDigite o valor buscado: ");
		scanf(" %d", x);
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
		pedir_valor(&x, 1);
		
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

main(){
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
	}while (op != 3);
}

