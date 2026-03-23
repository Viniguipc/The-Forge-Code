#include <stdio.h>
#include <conio.h>
#include <stdlib.h>

typedef struct no{
	int x;
	struct no* prox;
}lista;

void menu (int* op){
	do{
		printf("\tMENU\n");
		printf("\n----------");
		printf("\n1 - Inserir no Inicio\n2 - Inserir no Fim\n3 - Inserir no Meio\n4 - Excluir no Inicio\n5 - Excluir no Fim\n6 - Excluir no Meio\n7 - Sair");
		printf("\n----------\n");
		*op = _getche();
	}while (*op < 1 || *op > 7);
}

int main(){
	lista *inicio;
	int op;
	
	inicio = NULL;
	
	do{
		system("cls");
		menu(&op);
		
		
	}while (op != 7);
	
}
