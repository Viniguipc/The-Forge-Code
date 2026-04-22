/*Usando FILA, implemente a solução abaixo:
Considere um show de uma banda de rock, onde os ingressos foram vendidos de acordo com cada
área do estádio. Para cada tipo de ingresso, há uma área específica como apresentado da figura
abaixo:
• Ingresso Pista prêmium– vermelho – código: 1 (fila 1 leste/ Fila 1 oeste)
• Ingresso Pista – rosa – código: 2 (fila 2 leste/ Fila 2 oeste)
• Ingresso Cadeira gold – verde – código 3 (fila 3 leste/ Fila 3 oeste)
• Ingresso Cadeira – marrom – código 4 (fila 4 leste/ Fila 4 oeste)
São 4 tipos de ingressos possíveis. Sendo que para cada tipo de ingresso, há duas filas, uma na
entrada Leste e outra na entrada Oeste. Não há como o “Fã” saber qual entrada terá a menor fila.
Para isso, vamos otimizar essas filas por meio de um painel em que apresentará a quantidade de
pessoas que estão em cada uma das filas, assim, é possível escolher a fila para entrar, de acordo
com o ingresso de cada um.
Sempre que alguém entrar em uma das filas, você deve mostrar o “Painel das Filas”. Serão 8 filas
ao total. Sempre que alguém sair de uma fila, ou seja, conseguiu entrar no estádio, a fila é
atualizada e deve ser atualizado o “Painel das Filas”. Vamos supor que essa entrada e saída seja
feita por um sensor (mas no nosso caso real, vamos fazer a entrada e saída da fila via código).
Implemente cada uma das Filas:
a) Inserção Fila Pista Prêmium – Fila 1 leste/ Inserção Fila Pista Prêmium - Fila 1 oeste)
b) Inserção Fila Pista – Fila 2 leste/ Inserção Fila Pista -Fila 2 oeste)
c) Inserção Cadeira gold –Fila 3 leste/ Inserção Cadeira gold - Fila 3 oeste)
d) Inserção Cadeira - Fila 4 leste/ Inserção Cadeira - Fila 4 oeste)
e) Remoção para as opções de filas
f) Impressão de cada uma das filas
g) Impressão dos totais por fila (Visualização do Painel de Filas)
*/
#include <stdio.h>
#include <stdlib.h>

typedef struct _fila{
    int codigo;
    struct _fila* prox;
}Fila;

void imprimir_lista (Fila* inicio){
	Fila *pos;
	
	pos = inicio;
	
	printf("\n");
	
	if(inicio == NULL){
		printf("Lista Vazia\n");
	}
	else{
		while (pos != NULL){
			printf("%d", pos);
			pos = pos->prox;
			if (pos != NULL) {
				printf(" | ");
			}
		}
		printf("\n");
	}
}

void menu(int* op){
    do{
        printf("\n1 - Inserir Fila Pista Prêmium - Fila 1 leste\n");
        printf("2 - Inserir Fila Pista Prêmium - Fila 1 oeste\n");
        printf("3 - Inserir Fila Pista - Fila 2 leste\n");
        printf("4 - Inserir Fila Pista - Fila 2 oeste\n");
        printf("5 - Inserir Cadeira gold - Fila 3 leste\n");
        printf("6 - Inserir Cadeira gold - Fila 3 oeste\n");
        printf("7 - Inserir Cadeira - Fila 4 leste\n");
        printf("8 - Inserir Cadeira - Fila 4 oeste\n");
        printf("9 - Remover Fila Pista Prêmium - Fila 1 leste\n");
        printf("10 - Remover Fila Pista Prêmium - Fila 1 oeste\n");
        printf("11 - Remover Fila Pista - Fila 2 leste\n");
        printf("12 - Remover Fila Pista - Fila 2 oeste\n");
        printf("13 - Remover Cadeira gold - Fila 3 leste\n");
        printf("14 - Remover Cadeira gold - Fila 3 oeste\n");
        printf("15 - Remover Cadeira - Fila 4 leste\n");
        printf("16 - Remover Cadeira - Fila 4 oeste\n");
        printf("17 - Imprimir Fila Pista Prêmium - Fila 1 leste\n");
        printf("18 - Imprimir Fila Pista Prêmium - Fila 1 oeste\n");
        printf("19 - Imprimir Fila Pista - Fila 2 leste\n");
        printf("20 - Imprimir Fila Pista - Fila 2 oeste\n");
        printf("21 - Imprimir Cadeira gold - Fila 3 leste\n");
        printf("22 - Imprimir Cadeira gold - Fila 3 oeste\n");
        printf("23 - Imprimir Cadeira - Fila 4 leste\n");
        printf("24 - Imprimir Cadeira - Fila 4 oeste\n");
        printf("25 - Imprimir totais por fila\n");
        printf("0 - Sair\n");
        printf("Digite sua opcao: ");
        scanf("%d", op);
    }while(*op < 1 || *op > 25);
}

void excluir_inicio (Fila** inicio){
	Fila* pos;
	
	if(*inicio == NULL){
		printf("\nLista Vazia");		
	}
	else{
		pos = *inicio;
		*inicio = pos->prox;
		
		free(pos);
	}
}

void inserir_fim (Fila** inicio){
	Fila *pos, *novo;
	
	pos = *inicio;
	novo = (Fila*) malloc (sizeof(Fila));
	
	if(novo == NULL){
		printf("\nERRO");
	}
	else{
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

int contar_fila (Fila* inicio, Fila* fila_1_oeste, Fila* fila_2_leste, Fila* fila_2_oeste, Fila* fila_3_leste, Fila* fila_3_oeste, Fila* fila_4_leste, Fila* fila_4_oeste){
	int cont = 0;
	Fila* pos;

	pos = inicio;
	
	while(pos != NULL){
		cont++;
		pos = pos->prox;
	}

	pos = fila_1_oeste;
	
	while(pos != NULL){
		cont++;
		pos = pos->prox;
	}

	pos = fila_2_leste;
	
	while(pos != NULL){
		cont++;
		pos = pos->prox;
	}

	pos = fila_2_oeste;
	
	while(pos != NULL){
		cont++;
		pos = pos->prox;
	}

	pos = fila_3_leste;
	
	while(pos != NULL){
		cont++;
		pos = pos->prox;
	}

	pos = fila_3_oeste;
	
	while(pos != NULL){
		cont++;
		pos = pos->prox;
	}

	pos = fila_4_leste;
	
	while(pos != NULL){
		cont++;
		pos = pos->prox;
	}

	pos = fila_4_oeste;
	
	while(pos != NULL){
		cont++;
		pos = pos->prox;
	}
	
	return cont;
}

int main () {
    Fila *fila_1_leste, *fila_1_oeste, *fila_2_leste, *fila_2_oeste, *fila_3_leste, *fila_3_oeste, *fila_4_leste, *fila_4_oeste;
    int op;

    fila_1_leste = NULL;
    fila_1_oeste = NULL;
    fila_2_leste = NULL;
    fila_2_oeste = NULL;
    fila_3_leste = NULL;
    fila_3_oeste = NULL;
    fila_4_leste = NULL;
    fila_4_oeste = NULL;

    do{
        menu(&op);

        switch(op){
            case 1: 
                inserir_fim(&fila_1_leste);
                break;
            case 2: 
                inserir_fim(&fila_1_oeste);
                break;
            case 3: 
                inserir_fim(&fila_2_leste);
                break;
            case 4: 
                inserir_fim(&fila_2_oeste);
                break;
            case 5: 
                inserir_fim(&fila_3_leste);
                break;
            case 6: 
                inserir_fim(&fila_3_oeste);
                break;
            case 7: 
                inserir_fim(&fila_4_leste);
                break;
            case 8: 
                inserir_fim(&fila_4_oeste);
                break;
            case 9: 
                excluir_inicio(&fila_1_leste);
                break;
            case 10: 
                excluir_inicio(&fila_1_oeste);
                break;
            case 11: 
                excluir_inicio(&fila_2_leste);
                break;
            case 12: 
                excluir_inicio(&fila_2_oeste);
                break;
            case 13: 
                excluir_inicio(&fila_3_leste);
                break;
            case 14: 
                excluir_inicio(&fila_3_oeste);
                break;
            case 15: 
                excluir_inicio(&fila_4_leste);
                break;
            case 16: 
                excluir_inicio(&fila_4_oeste);
                break;
            case 17: 
                imprimir_lista(fila_1_leste);
                break;
            case 18: 
                imprimir_lista(fila_1_oeste);
                break;
            case 19: 
                imprimir_lista(fila_2_leste);
                break;
            case 20: 
                imprimir_lista(fila_2_oeste);
                break;
            case 21: 
                imprimir_lista(fila_3_leste);
                break;
            case 22: 
                imprimir_lista(fila_3_oeste);
                break;
            case 23: 
                imprimir_lista(fila_4_leste);
                break;
            case 24: 
                imprimir_lista(fila_4_oeste);
                break;
            case 25: 
                printf("\nTotal de pessoas nas filas: %d", contar_fila(fila_1_leste, fila_1_oeste, fila_2_leste, fila_2_oeste, fila_3_leste, fila_3_oeste, fila_4_leste, fila_4_oeste));
                break;
        }
    }while(op != 0);
}