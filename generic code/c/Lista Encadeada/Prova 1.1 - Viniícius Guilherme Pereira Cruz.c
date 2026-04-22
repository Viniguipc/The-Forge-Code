#include <stdio.h>
#include <stdlib.h>

typedef struct _doc{
	int dep;
	int car;
	int pri;
	int cod;
	int pri_final;
	struct _doc *prox;
}lista;

void valida_dep(int *dep){
	do{
		printf("\n\tDIGITE O DEPARTAMENTO\t");
		printf("\n1 - TI\n2 - FINANCEIRO\n3 - ADMINISTRATIVO\n4 - RH\nDigite o número do departamento: ");
		scanf(" %d", dep);
	}while(*dep < 1 || *dep > 4);
}

void valida_car(int *car){
	do{
		printf("\n\tDIGITE O CARGO\t");
		printf("\n1 - DEMAIS FUNCIONARIOS\n2 - GERENTE\n3 - DIRETOR\nDigite o número do cargo: ");
		scanf(" %d", car);
	}while(*car < 1 || *car > 3);
}

void valida_pri(int *pri){
	do{
		printf("\n\tDIGITE A PRIORIDADE\t");
		printf("\n1 - ROTINA\n5 - IMPORTANTE\n10 - URGENTE\nDigite o número da prioridade: ");
		scanf(" %d", pri);
	}while(*pri != 1 && *pri != 5 && *pri != 10);
}

void inserir_prioridade(lista** head, lista** feet, int cod){
	int dep, car, pri, pri_final;
	lista *novo, *aux, *ant;

	novo = (lista*)calloc(1, sizeof(lista));

	if(novo == NULL){
		printf("\nERRO\n");
	}
	else{
		valida_dep(&dep);
		valida_car(&car);
		valida_pri(&pri);

		pri_final = (car * 3) + (dep * 2) + pri;

		novo->cod = cod;
		novo->dep = dep;
		novo->car = car;
		novo->pri = pri;
		novo->pri_final = pri_final;
		
		if(*head == NULL){
			*head = novo;
			*feet = novo;

			novo->prox = NULL;
		}
		else{
			aux = *head;
			ant = NULL;

			while(aux != NULL && aux->pri_final > pri_final){
				ant = aux;
				aux = aux->prox;
			}

			if(aux == NULL){
				*feet = novo;
				ant->prox = novo;
				novo->prox = NULL;
			}
			else{
				if(ant == NULL){
					*head = novo;
					novo->prox = aux;
				}
				else{
					novo->prox = aux;
					ant->prox = novo;
				}
			}
		}
	}
}

void inserir_pilha(lista** head, lista* novo){
	lista *aux;

	aux = *head;
	*head = novo;
	novo->prox = aux;
}

void remover_cod(lista** head, lista** feet, lista** trash){
	
}

void imprimir(lista** head, lista** feet, lista** historic){
	lista *aux;

	if(*head == NULL){
		printf("\nLista Vazia");
	}
	else{
		aux = *head;
		*head = aux->prox;

		inserir_pilha(historic, aux);

		if(*head == NULL){
			*feet = NULL;
		}
	}
}

void listar(lista* head){
	lista *aux;
	
	aux = head;
	
	printf("\n");
	
	if(head == NULL){
		printf("\nLista Vazia");
	}
	else{
		while (aux != NULL){
			printf("\n%.7d | [%.5d] -> %.7d", aux, aux->cod, aux->prox);
			aux = aux->prox;
		}
	}
}

void consulta(lista* head){
	
}

void menu(int *op){
	do{
		printf("\n\n\tMENU\t");
		printf("\n1 - INSERIR NA FILA DE IMPRESSAO\n2 - CANCELAR IMPRESSÃO\n3 - CONSULTAR CODIGO\n4 - LISTAR FILA DE IMPRESSAO\n5 - LISTAR HISTORICO\n6 - LISTAR DESCARTE\n7 - IMPRIMIR\n8 - SAIR DO CODIGO\n");
		scanf(" %d", op);
	}while(*op < 1 || *op > 8);
}

int main(){
	lista *head, *feet, *trash, *historic;
	int op = 0, cod = 0;

	head = NULL;
	feet = NULL;
	trash = NULL;
	historic = NULL;

	while(op != 8){
		menu(&op);
		switch(op){
			case 1:
				inserir_prioridade(&head, &feet, cod);
				cod++;
				break;
			case 2:
				remover_cod(&head, &feet, &trash);
				break;
			case 3:
				consulta(head);
				break;
			case 4:
				listar(head);
				break;
			case 5:
				listar(historic);
				break;
			case 6:
				listar(trash);
				break;
			case 7:
				imprimir(&head, &feet, &historic);
				break;
		}
	}
}