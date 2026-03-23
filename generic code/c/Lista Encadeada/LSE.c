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
		printf("\n1 - Inserir no Inicio\n2 - Inserir no Fim\n3 - Inserir no Meio\n4 - Excluir no Inicio\n5 - Excluir no Fim\n6 - Excluir no Meio\n7 - Sair");
		printf("\n----------\n");
		*op = _getche() - '0';
	}while (*op < 1 || *op > 7);
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

void inserir_fim (lista** inicio){
	lista *pos, *novo;
	int x;
	
	pos = *inicio;
	novo = (lista*) malloc (sizeof(lista));
	
	if(novo == NULL){
		printf("\nERRO");
	}
	else{
		pedir_valor(&x, 1);
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

void inserir_meio (lista** inicio){
	lista *pos, *ant, *novo;
	int x, y;
	
	pos = *inicio;
	novo = (lista*) malloc (sizeof(lista));
	
	if(novo == NULL){
		printf("\nERRO");
	}
	else{
		pedir_valor(&y, 2);
		pedir_valor(&x, 1);
		novo->x = x;
		
		if(*inicio == NULL){
			novo->prox = *inicio;
			*inicio = novo;
		}
		else{
			while(pos->x != y && pos != NULL){
				ant = pos;
				pos = pos->prox;
			}
			
			if(pos == NULL){
				printf("\nValor não encontrado");
			}
			else{
				if(pos == *inicio){
					novo->prox = *inicio;
					*inicio = novo;	
				}
				else{
					ant->prox = novo;
					novo->prox = pos;	
				}
			}
		}
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

void excluir_fim (lista** inicio){
	lista *pos, *aux;
	
	if(*inicio == NULL){
		printf("\nLista Vazia");
	}
	else{
		pos = *inicio;
		
		while(pos->prox != NULL){
			aux = pos;
			pos = pos->prox;
		}

		aux->prox = NULL;
		free(pos);
	}
}

void excluir_meio (lista** inicio){
	lista *pos, *aux;
	int y;
	
	if(*inicio == NULL){
		printf("\nLista Vazia");
	}
	else{
		pedir_valor(&y, 2);
		pos = *inicio;
		
		while(pos->x != y && pos != NULL){
			aux = pos;
			pos = pos->prox;
		}
		if(pos == NULL){
			printf("\nValor não encontrado");
		}
		else{
			if(pos == *inicio){
				*inicio = pos->prox;
		
				free(pos);
			}
			else{
				if(pos->prox == NULL){
					aux->prox = NULL;
					free(pos);
				}
				else{
					aux->prox = pos->prox;
					free(pos);
				}
			}
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
				inserir_meio(&inicio);
				break;
			case 4:
				excluir_inicio(&inicio);
				break;
			case 5:
				excluir_fim(&inicio);
				break;
			case 6:
				excluir_meio(&inicio);
				break;
		}
	}while (op != 7);
	
}
