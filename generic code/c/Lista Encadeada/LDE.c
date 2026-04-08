#include <stdio.h>
#include <stdlib.h>
#include "raylib.h"

typedef struct no{
	struct no *ant;
	struct no *prox;
	int x;
}lista;

void imprimir (lista* head){
	if(head == NULL){
		printf("\nLista vazia\n");
	}
	else{
		while(head != NULL){
			printf("%d <- [%d] *%d -> %d", head->ant, head->x, head, head->ant);
			head = head->prox;
		}
	}
}

void menu (int* op){
	do{
		printf("\n1 - Inserir no Inicio\n2 - Inserir no Fim\n3 - Sair ");
		scanf(" %d", op);
	}while(*op < 1 || *op > 3);
}

int main(){
	lista *head, *feet;
	int op;
	
	head = feet = NULL;
	
	imprimir(head);
	menu(&op);	
}