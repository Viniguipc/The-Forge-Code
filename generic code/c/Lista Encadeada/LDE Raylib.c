#include <stdio.h>
#include <stdlib.h>
#include "raylib.h"

typedef struct no{
	struct no *ant;
	struct no *prox;
	int x;
}lista;

int valor(int* x){
	static char charnum[10];
	static int cont = 0, digitando = 0, digito;
	
	DrawText("Digite o valor desejado: ", (GetScreenWidth() / 2) - (MeasureText("Digite o valor desejado: ", GetScreenHeight() / 40) / 2), 200, GetScreenHeight() / 40, LIME);
	
	if(cont < 10){
		digito = GetCharPressed();
		if(digito >= '0' && digito <= '9'){
			charnum[cont] = digito;
			cont++;
		}
		else{
			if(IsKeyPressed(KEY_BACKSPACE)){
				cont--;
				if(cont < 0) cont = 0;
				charnum[cont] = '\0';
			}
		}
	}
	
	if(IsKeyPressed(KEY_ENTER)){
		*x = atoi(charnum);
		return 0;
	}
	
	return 1;
}

void inserir_inicio(lista** head, lista** feet, int x){
	lista *novo;
	
	novo = (lista*) malloc (sizeof(struct no));
	
	if(novo == NULL){
		DrawText("ERRO", ((GetScreenWidth() / 2) - (MeasureText("ERRO", 20) / 2)), 500, 20, BLUE);
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
		DrawText("ERRO", ((GetScreenWidth() / 2) - (MeasureText("ERRO", 20) / 2)), 450, 20, BLUE);
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

void inserir_meio(lista** head, lista** feet, int x){
	lista* aux, *novo;
	
	aux = *head;
	
	while(aux != NULL && aux->x != x){
	aux = aux->prox;
	}
	
	if(aux == NULL){
		DrawText("Valor não encontrado", ((GetScreenWidth() / 2) - (MeasureText("Valor não encontrado", 20) / 2)), 450, 20, BLUE);
	}
	else{
		if(aux->ant == NULL){
			inserir_inicio(head, feet, x);
		}
		else{
			if(aux->prox == NULL){
				inserir_fim(head, feet, x);
			}
			else{
				novo = (lista*) malloc (sizeof(struct no));

				if(novo == NULL){
					DrawText("ERRO", ((GetScreenWidth() / 2) - (MeasureText("ERRO", 20) / 2)), 500, 20, BLUE);
				}
				
				novo->ant = aux->ant;
				novo->prox = aux;
				novo->x = x;
				
				aux->ant->prox = novo;
				aux->ant = novo;
			}
		}
	}
}


void excluir_inicio (lista** head, lista** feet){
	lista *pos;
	if(*head == NULL){
		DrawText("LISTA VAZIA", ((GetScreenWidth() / 2) - (MeasureText("LISTA VAZIA", 20) / 2)), 450, 20, BLUE);
	}
	else{
		pos = *head;
		
		*head = pos->prox;
		if(*head == NULL){
			*feet = NULL;
		}
		else{
			(*head)->ant = NULL;
		}
		free(pos);
	}
}

void excluir_fim (lista** head, lista **feet){
	lista *pos;
	if(*head == NULL){
		DrawText("LISTA VAZIA", ((GetScreenWidth() / 2) - (MeasureText("LISTA VAZIA", 20) / 2)), 450, 20, BLUE);
	}
	else{
		pos = *feet;
		
		*feet = pos->ant;
		if(*feet == NULL){
			*head = NULL;
		}
		else{
			(*feet)->prox = NULL;
		}
		
		free(pos);
	}
}

void excluir_meio (lista** head, lista** feet, int x){
	lista *pos, *aux;
	
	if(*head == NULL){
		DrawText("LISTA VAZIA", ((GetScreenWidth() / 2) - (MeasureText("LISTA VAZIA", 20) / 2)), 450, 20, BLUE);
	}
	else{
		pos = *head;
		
		while(pos != NULL && pos->x != x){
			pos = pos->prox;
		}
		
		if(pos == NULL){
			DrawText("Valor não encontrado", ((GetScreenWidth() / 2) - (MeasureText("Valor não encontrado", 20) / 2)), 450, 20, BLUE);
		}
		else{
			if(pos->ant == NULL){
				excluir_inicio(head, feet);
			}
			else{		
				if(pos->prox == NULL){
					excluir_fim(head, feet);
				}
				else{
					aux = pos->prox;
					aux->ant = pos->ant;
					aux = pos->ant;
					aux->prox = pos->prox;
				}
			}
		}
	}
}

void imprimir (lista* head){
	int i;
	while(head != NULL){
		const char* texto = TextFormat("%p <- [%d] *%p -> %p", head->ant, head->x, head, head->prox);
		DrawText(texto, ((GetScreenWidth() / 2) - (MeasureText(texto, 20) / 2)), 100 + (50 * i), 20, LIGHTGRAY);
		head = head->prox;
		i++;
	}
}

void menu (int* op, int* digitando){
	char *opcoes[] = {"1 - Inserir no Inicio", "2 - Inserir no Fim", "3 - Inserir no Meio", "4 - Excluir no Inicio", "5 - Excluir no Fim", "6 - Excluir no meio", "7 - Sair"};
	static int escolha = 1;
	int i;
	
	if(IsKeyPressed(KEY_ENTER)){
		*op = escolha;
		*digitando = 1;
	}
	else{
		if(IsKeyPressed(KEY_DOWN)){
			(escolha)++;
			if(escolha > 7){
				escolha = 1;
			}
		}
		
		if(IsKeyPressed(KEY_UP)){
			(escolha)--;
			if(escolha < 1){
				escolha = 7;
			}
		}
	}
	
	for(i = 0; i < 7; i++){
		if((i + 1) != escolha){
			DrawText(opcoes[i], (GetScreenWidth() / 2) - (MeasureText(opcoes[i], GetScreenHeight() / 40) / 2), 50 * (i + 1) + (GetScreenHeight() / 2), GetScreenHeight() / 40, WHITE);
		}
		else{
			DrawText(opcoes[i], (GetScreenWidth() / 2) - (MeasureText(opcoes[i], GetScreenHeight() / 40) / 2), 50 * (i + 1) + (GetScreenHeight() / 2), GetScreenHeight() / 40, RED);
		}
	}
}

int main(){
	lista *head, *feet;
	int op = 0, x = 0, digitando = 0, busca = 0;
	
	head = feet = NULL;
	
	SetConfigFlags(FLAG_WINDOW_UNDECORATED);
	InitWindow(GetMonitorWidth(0), GetMonitorHeight(0), "Lista Duplamente Encadeada");
	SetTargetFPS(60);
	
	while(!WindowShouldClose() && op != 7){
		x = 0;
		
		BeginDrawing();
			ClearBackground(BLACK);
			
			DrawText("Lista Atualizada", ((GetScreenWidth() / 2) - (MeasureText("Lista Atualizada", GetScreenHeight() / 20) / 2)), 50, GetScreenHeight() / 20, RED);
			
			if(head == NULL){
				DrawText("Lista Vazia", ((GetScreenWidth() / 2) - (MeasureText("Lista Vazia", GetScreenHeight() / 50) / 2)), 100, GetScreenHeight() / 50, LIGHTGRAY);
			}
			else{
				if(busca = 0){
					imprimir(head);
				}
			}
			
			if(digitando == 0){
				DrawText("MENU", ((GetScreenWidth() / 2) - (MeasureText("MENU", GetScreenHeight() / 25) / 2)), (GetScreenHeight() / 2), GetScreenHeight() / 25, LIGHTGRAY);
				menu(&op, &digitando);
			}
			else{
				switch(op){
					case 1:
						if(busca == 1){
							busca = valor(&x);	
						}

						inserir_inicio(&head, &feet, x);
						op = 0;
						digitando = 0;
						break;
					case 2:
						inserir_fim(&head, &feet, x);
						op = 0;
						digitando = 0;
						break;
					case 3:
						op = 0;
						digitando = 0;
						break;
					case 4:
						excluir_inicio(&head, &feet);
						op = 0;
						digitando = 0;
						break;
					case 5:
						excluir_fim(&head, &feet);
						op = 0;
						digitando = 0;
						break;
					case 6:
						excluir_meio(&head, &feet, x);
						op = 0;
						digitando = 0;
						break;			
				}
			}
		EndDrawing();
	}
	CloseWindow();
}