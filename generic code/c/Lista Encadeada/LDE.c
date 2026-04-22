#include <stdio.h>
#include <stdlib.h>

typedef struct no{
	struct no *ant;
	struct no *prox;
	int x;
}lista;

void inserir_inicio(lista** head, lista** feet, int x){
	lista *novo;
	
	novo = (lista*) malloc (sizeof(struct no));
	
	if(novo == NULL){
		printf("\nERRO");
	}
	else{
		novo->x = x;

		if(*head == NULL){
			novo->ant = NULL;
			novo->prox = NULL;
			*head = novo;
			*feet = novo;
		}
		else{
			novo->ant = NULL;
			novo->prox = *head;
			(*head)->ant = novo;
			*head = novo;
		}
	}
}

void inserir_fim (lista** head, lista** feet, int x){
	lista *novo;
	
	novo = (lista*) malloc (sizeof(struct no));
	
	if(novo == NULL){
		printf("\nERRO");
	}
	else{
		novo->x = x;
		novo->prox = NULL;
		novo->ant = *feet;
		
		if(*head == NULL){
			*head = novo;
		}
		else{
			(*feet)->prox = novo;
		}
		*feet = novo;
	}
}

void imprimir (lista* head){
	if(head == NULL){
		printf("\nLista vazia\n");
	}
	else{
		while(head != NULL){
			printf("\n%d <- [%d] *%d -> %d", head->ant, head->x, head, head->prox);
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
	int op, x;
	
	head = feet = NULL;
	
	do{
		imprimir(head);
		menu(&op);
		switch(op){
			case 1:
				printf("\nInsira o valor: ");
				scanf(" %d", &x);
				inserir_inicio(&head, &feet, x);
				break;
			case 2: 
				printf("\nInsera o valor: ");
				scanf(" %d", &x);
				inserir_fim(&head, &feet, x);
				break;
		}
	}while(op != 3);
}