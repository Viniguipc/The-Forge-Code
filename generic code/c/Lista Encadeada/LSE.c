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
	
	while (pos != NULL){
		printf("\n%d | [%d] -> %d", pos, pos->x, pos->prox);
		pos = pos->prox;
	}
}

void menu (int* op){
	do{
		printf("\tMENU\n");
		printf("\n----------");
		printf("\n1 - Inserir no Inicio\n2 - Inserir no Fim\n3 - Inserir no Meio\n4 - Excluir no Inicio\n5 - Excluir no Fim\n6 - Excluir no Meio\n7 - Sair");
		printf("\n----------\n");
		*op = _getche();
	}while (*op < 1 || *op > 7);
}

void inserir_inicio (lista** inicio){
	
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
		}
	}while (op != 7);
	
}
