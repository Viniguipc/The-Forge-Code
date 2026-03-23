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
	
	while (pos != NULL){
		printf("\n%d | [%d] -> %d", pos, pos->x, pos->prox);
		pos = pos->prox;
	}
}

void menu (int* op){
	do{
		printf("\n\n\tMENU\n");
		printf("\n----------");
		printf("\n1 - Inserir no Inicio\n2 - Inserir no Fim\n3 - Inserir no Meio\n4 - Excluir no Inicio\n5 - Excluir no Fim\n6 - Excluir no Meio\n7 - Sair");
		printf("\n----------\n");
		*op = _getche() - '0';
	}while (*op < 1 || *op > 7);
}

void pedir_valor(int* x){
	printf("\nDigite o valor: ");
	scanf(" %d", x);
}

void inserir_inicio (lista** inicio){
	int x;
	lista *novo;
	
	novo = (lista*) malloc (sizeof(lista));
	
	if(novo == NULL){
		printf("\nERRO");
	}
	else{
		pedir_valor(&x);
		
		novo->x = x;
		novo->prox = *inicio;
		*inicio = novo;
	}
}

void inserir_fim (lista** inicio){
	lista *pos, *novo;
	int x;
	
	pos = *inicio;
	novo = (lista*) malloc (sizeof(lista));
	
	if(novo == NULL){
		printf("\nERRO");
	}
	else{
		pedir_valor(&x);
		novo->x = x;
		
		if(*inicio == NULL){
			novo->prox = *inicio;
			*inicio = novo;
		}
		else{
			while(pos->prox != NULL){
				pos = pos->prox;
			}
			pos->prox = novo;
			novo->prox = NULL;
		}
	}
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
				inserir_fim(&inicio);
				break;
			case 3:
				break;
			case 4:
				break;
			case 5:
				break;
			case 6:
				break;
		}
	}while (op != 7);
	
}
