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
		printf("\n\tMENU\n");
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
	
	novo = (lista*) malloc (sizeof(lista*));
	
	pedir_valor(&x);
	
	novo->x = x;
	novo->prox = *inicio;
	*inicio = novo;
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
