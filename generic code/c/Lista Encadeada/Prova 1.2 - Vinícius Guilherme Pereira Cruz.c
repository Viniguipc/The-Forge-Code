#include <stdio.h>
#include <stdlib.h>

typedef struct no{
	struct no *ant;
	struct no *prox;
	int ip;
}lista;

void inserir_inicio(lista** head, lista** feet, int ip){
	lista *novo;
	
	novo = (lista*) malloc (sizeof(struct no));
	
	if(novo == NULL){
		printf("\nERRO");
	}
	else{
		novo->ip = ip;

		if(*head == NULL){
			novo->ant = novo;
			novo->prox = novo;
			*head = novo;
			*feet = novo;
		}
		else{
			novo->ant = *feet;
			novo->prox = *head;
			(*head)->ant = novo;
			*head = novo;
            (*feet)->prox = novo;
		}
	}
}

void inserir_fim (lista** head, lista** feet, int ip){
	lista *novo;
	
	novo = (lista*) malloc (sizeof(struct no));
	
	if(novo == NULL){
		printf("\nERRO");
	}
	else{
		novo->ip = ip;
		novo->ant = *feet;
		
		if(*head == NULL){
			*head = novo;
		}
		else{
			(*feet)->prox = novo;
		}

		novo->prox = *head;
        (*head)->ant = novo;
		*feet = novo;
	}
}

void pesquisar_ip(lista** head, lista** feet, int ip){
	lista *aux;

	if(*head == NULL){
		printf("\nLista vazia\n");
	}
	else{
		aux = *head;
		while(aux != NULL && aux->ip != ip){
			aux = aux->prox;
		}
		if(aux == NULL){
			printf("\nIP nao encontrado\n");
		}
		else{
			printf("\nIP encontrado -> Endereço: %d\n", aux);
		}
	}
}

void imprimir (lista* head, lista* feet){
    lista *aux;

    aux = head;

	if(head == NULL){
		printf("\nLista vazia\n");
	}
	else{
		do{
			printf("\n%d <- [%d] *%d -> %d", aux->ant, aux->ip, aux, aux->prox);
			aux = aux->prox;
		}while(aux != head);

        printf("\n\nInicio -> %d | Fim -> %d\n", head, feet);
	}
}

void menu (int* op){
	do{
		printf("\n1 - Inserir no Inicio\n2 - Inserir no Fim\n3 - Imprimir\n4 - Pesquisar IP\n5 - Sair ");
		scanf(" %d", op);
	}while(*op < 1 || *op > 5);
}

int main(){
	lista *head, *feet;
	int op, x;
	
	head = feet = NULL;
	
	do{
		menu(&op);
		switch(op){
			case 1:
				x = rand() % 100;
				inserir_inicio(&head, &feet, x);
				break;
			case 2: 
				x = rand() % 100;
				inserir_fim(&head, &feet, x);
				break;
            case 3:
                imprimir(head, feet);
                break;
            case 4:
                printf("\nInsira o valor: ");
                scanf(" %d", &x);
                pesquisar_ip(&head, &feet, x);
                break;
		}
	}while(op != 5);
}