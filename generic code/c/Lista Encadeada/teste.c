#include <stdio.h>
#include <stdlib.h>

typedef struct _doc{
	int dep;
	int car;
	int pri;
	int cod;
	int tempo;
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

			while(aux != NULL && aux->pri_final >= pri_final){
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

void consulta(lista* head, int cod){
	lista* aux;

	if(head == NULL){
		printf("\nLista Vazia");
	}
	else{
		aux = head;
		
		while(aux != NULL && aux->cod != cod){
			aux = aux->prox;
		}

		if(aux == NULL){
			printf("\nCodigo nao encontrado\n");
		}
		else{
			printf("\n%.7d | [%.5d] -> %.7d\n", aux, aux->cod, aux->prox);
			switch(aux->dep){
				case 1:
					printf("DEPARTAMENTO: TI\n");
					break;
				case 2:
					printf("DEPARTAMENTO: FINANCEIRO\n");
					break;
				case 3:
					printf("DEPARTAMENTO: ADMINISTRATIVO\n");
					break;
				case 4:
					printf("DEPARTAMENTO: RH\n");
					break;
			}
			switch(aux->car){
				case 1:
					printf("CARGO: DEMAIS FUNCIONARIOS\n");
					break;
				case 2:
					printf("CARGO: GERENTE\n");
					break;
				case 3:
					printf("CARGO: DIRETOR\n");
					break;
			}
			switch(aux->pri){
				case 1:
					printf("PRIORIDADE: ROTINA\n");
					break;
				case 5:
					printf("PRIORIDADE: IMPORTANTE\n");
					break;
				case 10:
					printf("PRIORIDADE: URGENTE\n");
					break;
			}
			printf("PRIORIDADE FINAL: %d\n", aux->pri_final);
			printf("TEMPO NA FILA: %d\n", aux->tempo);
		}
	}
}

void menu(int *op){
	do{
		printf("\n\n\tMENU\t");
		printf("\n1 - INSERIR NA FILA DE IMPRESSAO\n2 - CANCELAR IMPRESSÃO\n3 - CONSULTAR CODIGO\n4 - LISTAR FILA DE IMPRESSAO\n5 - LISTAR HISTORICO\n6 - LISTAR DESCARTE\n7 - IMPRIMIR\n8 - SAIR DO CODIGO\n");
		scanf(" %d", op);
	}while(*op < 1 || *op > 8);
}

void organizar_fila(lista** head, lista** feet, lista* atual){
	lista *aux, *ant;

	if(*head == NULL){
		*head = atual;
		*feet = atual;
		atual->prox = NULL;
	}
	else{
		aux = *head;
		ant = NULL;
		
		while(aux != NULL && aux->pri_final >= atual->pri_final){
			ant = aux;
			aux = aux->prox;
		}
		
		if(aux == NULL){
			*feet = atual;
			ant->prox = atual;
			atual->prox = NULL;
		}
		else{
			if(ant == NULL){
				*head = atual;
				atual->prox = aux;
			}
			else{
				ant->prox = atual;
				atual->prox = aux;
			}
		}
	}
}

void aumentar_tempo(lista** head, lista** feet){
	lista *aux, *ant, *organizar;

	if(*head == NULL){

	}
	else{
		aux = *head;
		ant = NULL;
		
		while(aux != NULL){
			aux->tempo++;
			if((aux->tempo % 5) == 0){
				aux->pri_final += 1;
				if(ant == NULL){
					ant = aux;
					aux = aux->prox;
				}
				else{
					ant->prox = aux->prox;
					organizar_fila(head, feet, aux);
					aux = ant->prox;
				}
			}
			else{
				aux = aux->prox;
			}
		}
	}
}

int main(){
	lista *head, *feet, *trash, *historic;
	int op = 0, cod = 0, buscar;

	head = NULL;
	feet = NULL;
	trash = NULL;
	historic = NULL;

	while(op != 8){
		menu(&op);
		aumentar_tempo(&head, &feet);
		switch(op){
			case 1:
				inserir_prioridade(&head, &feet, cod);
				cod++;
				break;
			case 2:
				remover_cod(&head, &feet, &trash);
				break;
			case 3:
				do{
					printf("\nDIGITE O CODIGO A SER CONSULTADO: ");
					scanf(" %d", &buscar);
				}while(buscar < 0);
				consulta(head, buscar);
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
	return 0;
}