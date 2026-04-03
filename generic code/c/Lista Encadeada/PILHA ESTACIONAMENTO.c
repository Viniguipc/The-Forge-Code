#include <stdio.h>
#include <conio.h>
#include <stdlib.h>
#include <string.h>
#include <windows.h>

typedef struct no{
	char carro[8];
	struct no* prox;
}lista;

void imprimir_lista_rec(lista *inicio_park, lista *inicio_rua) {
	if (inicio_park == NULL && inicio_rua == NULL) {
		printf("\n==============\t========");
	}
	else{
			if (inicio_park == NULL) {
			printf("\n|            |\t");
		} else {
			printf("\n| [%s] |\t", inicio_park->carro);
			inicio_park = inicio_park->prox;
		}
		
		if (inicio_rua == NULL) {
			printf("|            |");
		} else {
			printf("| [%s] |", inicio_rua->carro);
			inicio_rua = inicio_rua->prox;
		}
		
		imprimir_lista_rec(inicio_park, inicio_rua);
	}
}

void imprimir_lista (lista* inicio_park, lista* inicio_rua){
	lista *pos_park, *pos_rua;
	
	pos_park = inicio_park;
	pos_rua = inicio_rua;
	
	printf("\n");
	
	if(inicio_park == NULL && inicio_rua){
		printf("\nLista Vazia");
	}
	else{
		printf("\nESTACIONAMENTO\tMEIO-FIO");
		printf("\n==============\t========");
		
		while (pos_park != NULL || pos_rua){
			if(pos_park == NULL){
				printf("\n|           |\t");
			}
			else{
				printf("\n| [%s] |\t", pos_park->carro);
				pos_park = pos_park->prox;
			}
			
			if(pos_rua == NULL){
				printf("|           |");
			}
			else{
				printf("| [%s] |", pos_rua->carro);
				pos_rua = pos_rua->prox;
			}
		}
		
		printf("\n==============\t========");
	}
}

void menu (int* op){
	do{
		printf("\n\n\tMENU\n");
		printf("\n----------");
		printf("\n1 - Estacionar Carro\n2 - Retirar Carro\n3 - Sair");
		printf("\n----------\n");
		*op = _getche() - '0';
	}while (*op < 1 || *op > 3);
}

void pedir_valor(char* placa){
	printf("\nDigite a Placa (CAR-XXX): ");
	scanf(" %s", placa);
}

void inserir_inicio (lista** inicio_park, int cont){
	lista *novo;
	
	novo = (lista*) malloc (sizeof(lista));
	
	if(novo == NULL){
		printf("\nERRO");
	}
	else{
		sprintf(novo->carro, "CAR-%03d", cont);

		novo->prox = *inicio_park;
		*inicio_park = novo;
	}
}

void retirar_pilha (lista** inicio_park, lista** inicio_rua){
	char placa[8];
	lista *pos;
	int encontrou = 0;
	
	if(*inicio_park == NULL){
		printf("\nEstacionamento Vazio");
	}
	else{
		pedir_valor(placa);
		
		while(*inicio_park != NULL && encontrou != 1){
			if(strcmp((*inicio_park)->carro, placa) != 0){
				pos = *inicio_park;
				*inicio_park = pos->prox;
				pos->prox = *inicio_rua;
				*inicio_rua = pos;
				
				system("cls");
				imprimir_lista(*inicio_park, *inicio_rua);
				printf("\nManobrando %s para a rua...", pos->carro);
				Sleep(800);
			}
			else{
				pos = *inicio_park;
				*inicio_park = pos->prox;
				
				free(pos);
				encontrou = 1;
			}
		}
		
		if(encontrou == 0){
			printf("\nERRO: Carro não encontrado! ");
			
			Sleep(1800);
		}
		else{
			Sleep(800);
			
			while(*inicio_rua != NULL){
				pos = *inicio_rua;
				*inicio_rua = pos->prox;
				pos->prox = *inicio_park;
				*inicio_park = pos;
				
				system("cls");
				imprimir_lista(*inicio_park, *inicio_rua);
				printf("\nDevolvendo %s ao estacionamento...", pos->carro);
				Sleep(800);
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

main(){
		lista *inicio_park, *inicio_rua;
	int op, cont = 0;
	
	inicio_park = inicio_rua = NULL;
	
	do{
		system("cls");
		
		imprimir_lista(inicio_park, inicio_rua);
		menu(&op);
		
		switch(op){
			case 1:
				inserir_inicio(&inicio_park, cont);
				cont++;
				break;
			case 2:
				retirar_pilha(&inicio_park, &inicio_rua);
				break;
		}
	}while (op != 3);
}

