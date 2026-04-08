#include <stdio.h>
#include <stdlib.h>
#include "raylib.h"

typedef struct no{
	struct no *ant;
	struct no *prox;
	int x;
}lista;

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
			DrawText(opcoes[i], 250, 50 * (i + 2) + 450, 30 , WHITE);
		}
		else{
			DrawText(opcoes[i], 250, 50 * (i + 2) + 450, 30, RED);
		}
	}
}

int main(){
	lista *head, *feet;
	int op = 0, x = 0, digitando = 0;
	
	head = feet = NULL;
	
	InitWindow(1200, 900, "Lista Duplamente Encadeada");
	SetTargetFPS(60);
	
	while(!WindowShouldClose() && op != 7){
		BeginDrawing();
			ClearBackground(BLACK);
			
			DrawText("Lista Atualizada", ((GetScreenWidth() / 2) - (MeasureText("Lista Atualizada", 40) / 2)), 50, 40, RED);
			
			if(head == NULL){
				DrawText("Lista Vazia", ((GetScreenWidth() / 2) - (MeasureText("Lista Vazia", 20) / 2)), 100, 20, LIGHTGRAY);
			}
			else{
				imprimir(head);
			}
			
			if(digitando == 0){
				DrawText("MENU", ((GetScreenWidth() / 2) - (MeasureText("MENU", 20) / 2)), 500, 20, LIGHTGRAY);
				menu(&op, &digitando);
			}
			else{
				switch(op){
					case 1:
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
						op = 0;
						digitando = 0;
						break;
					case 6:
						op = 0;
						digitando = 0;
						break;			
				}
			}
		EndDrawing();
	}
	
	CloseWindow();
}