#include <stdio.h>
#include <stdlib.h>
#include <string.h>


#define IS_COMMAND(car) (car=='+' || car=='-' || car=='>' || car=='<' || car=='.' || car==',' || car=='[' || car==']')

#define CROCHET(car) (car=='[' || car==']')








typedef struct Node {
    int data;
    struct Node* next;
} Node;

typedef struct {
    Node* top;
} Stack;





// Structure d'un élément de la file
typedef struct NodeQ {
    char data;
    struct NodeQ* prev;
    struct NodeQ* next;
} NodeQ;

// Structure de la file
typedef struct {
    NodeQ* front;
    NodeQ* rear;
} Queue;

// Fonction pour créer une nouvelle file vide
Queue* createQueue(void) {
    Queue* queue = (Queue*)malloc(sizeof(Queue));
    if (queue == NULL) {
        printf("Memory error\n");
        exit(1);
    }
    queue->front = queue->rear = NULL;
    return queue;
}

// Fonction pour vérifier si la file est vide
int isEmptyQ(Queue* queue) {
    return (queue->front == NULL);
}

// Fonction pour ajouter un élément à l'arrière de la file
void enqueue(Queue* queue, char data) {
    NodeQ* newNode = (NodeQ*)malloc(sizeof(NodeQ));
    if (newNode == NULL) {
        printf("Memory error\n");
        exit(1);
    }
    newNode->data = data;
    newNode->next = NULL;

    if (isEmptyQ(queue)) {
        newNode->prev = NULL;
        queue->front = queue->rear = newNode;
    } else {
        newNode->prev = queue->rear;
        queue->rear->next = newNode;
        queue->rear = newNode;
    }
}

// Fonction pour retirer un élément de l'avant de la file
int dequeue(Queue* queue) {
    if (isEmptyQ(queue)) {
        printf("Error : Empty queue\n");
        exit(1);
    }
    NodeQ* temp = queue->front;
    int data = temp->data;

    if (queue->front == queue->rear) {
        queue->front = queue->rear = NULL;
    } else {
        queue->front = queue->front->next;
        queue->front->prev = NULL;
    }

    free(temp);
    return data;
}





void initialize(Stack* stack) {
    stack->top = NULL;
}


void push(Stack* stack, int value) {
    Node* newNode = (Node*)malloc(sizeof(Node));
    if (newNode == NULL) {
        printf("Memory allocation failed!\n");
        return;
    }
    newNode->data = value;
    newNode->next = stack->top;
    stack->top = newNode;
}



int isEmpty(Stack* stack) {
    return stack->top == NULL;
}

int pop(Stack* stack) {
    if (isEmpty(stack)) {
        printf("Stack is empty!\n");
        return -1; // Return a sentinel value or handle empty stack as needed
    }
    Node* poppedNode = stack->top;
    int poppedValue = poppedNode->data;
    stack->top = poppedNode->next;
    free(poppedNode);
    return poppedValue;
}







void cleanStdin(void)// vide le buffer
{
    int c = 0;
    while ((c = getchar()) != '\n' && c != EOF);
}





char* input(unsigned *len)
{
  char* var=malloc(3000*sizeof(char)); // allocation d'un pointeur pour l'entrée de l'utilisateur (+1 char pour le caractère nul)
  memset(var,(char)0,3000*sizeof(char));//initialise le pointeur à '\0' partout
  //on effectue l'entrée

  int err = scanf("%2999[^\n]",var);

  cleanStdin();

  if (err!=1)
  {
	free(var);
    return strdup("");
  }

  *len = strlen(var);
  //crée un deuxième pointeur pour y copier le contenu de l'entrée de la vraie longueur
  char* newVar = malloc(sizeof(char)*(*len+1));//réserve une place de la longueur de l'entrée + 1 pour le caractère nul

  void * ptrtest=strcpy(newVar,var);//copie de var dans newVar

  free(var);
  var=NULL;
    
  if (ptrtest==NULL)
  {
    free(newVar);
    printf("Erreur\n");
    return NULL;
  }
  
  return newVar;
}




char* loadProgram(char* name, unsigned* longueur)
{
  *longueur = 0;
  
  FILE* fichier = fopen(name, "rt");//lit le fichier
  
  if (fichier==NULL)
  {
    printf("Erreur lors de l'ouverture du fichier.\n");
    return NULL;
  }

  
  char car;//le caractère que l'on va étudier

  
  //on regarde la position actuelle de la tête de fichier
  fpos_t pos;
  fgetpos(fichier, &pos);

  
  // calcul de la longueur du programme
  char carAnc=0;
  while ((car=fgetc(fichier))!=EOF)
  {
    if IS_COMMAND(car)//on ne compte que les commandes
    {
      if CROCHET(car)
        *longueur+=2;
      
      else if (carAnc!=car)
        *longueur+=2;
      
    }
    carAnc=car;
  }

  
  fsetpos(fichier, &pos);//remet la tête de fichier au début
  char* program=malloc(*longueur+1);// crée le tableau de caractères qui va contenir le programme

  //copier le programme dans un tableau
  unsigned i=0;
  carAnc=(char)0;
  while ((car=fgetc(fichier))!=EOF)
  {
    if IS_COMMAND(car)//on ne compte que les commandes
    {
      if CROCHET(car)//si on a un crochet
      {
        program[i]=car;//copie du crochet
        program[i+1]=(char)1;//1 dans le paramètre
        i+=2;
      }
      else
      {
        if (carAnc==car)//si le caractère est le même que le précédent
          program[i-1]++;
        
        else
        {
          program[i]=car;//nouveau caractère
          program[i+1]=(char)1;
          i+=2;
        }
      }
    }
    carAnc=car;
  }
  
  program[*longueur]='\0';

  fclose(fichier);
  
  return program;
}




int crochets(char* program, unsigned longueur, unsigned* crochOuv, unsigned* crochFerm)
{
  unsigned index=0;
  unsigned nbCrochets=0;

  Stack stack0;
  initialize(&stack0);

  //met en place une pile qui compte les crochets, et à chaque dépilage, on ajoute les index du programme correspondants

  
  while (index<longueur)
  {
    
    if (program[index]=='[')
    {
      push(&stack0, index);
    }
    if (program[index]==']')
    {
      int el = pop(&stack0);
      crochOuv[el]=index;
      crochFerm[index]=el;
    }

    index+=2;
  }
  return !isEmpty(&stack0);
  
}









void virgule(Queue* bande_entree, char* program, unsigned numProg, char* memoire, unsigned ptr)
{
    for (unsigned i=0;i<program[numProg+1];i++)
    {
        if (isEmptyQ(bande_entree))
        {
            unsigned len;
            char* entree = input(&len);
            
            memoire[ptr] = entree[0];
            
            for (register unsigned i = 1 ; i < len ; i++)
                enqueue(bande_entree, entree[i]);
        
        }
        else
        {
            memoire[ptr] = dequeue(bande_entree);
        }

    }
}



/*
4 types de déplacements possibles :
[->+<]
[>+<-]
[-<+>]
[<+>-]
*/




void optimize(char* prog, unsigned len)
{
    char* temp;
    for (int i = 0 ; i < len - 12; i+=2) // 12 est la taille d'un déplacement dans la mémoire
    {
        // pour simplifier la comparaison
        temp = malloc(sizeof(char)*7);
        for (unsigned j=0 ; j < 6 ; j++)
            temp[j] = prog[i+2*j];
        temp[6] = '\0';
        
        if (strcmp(temp, "[->+<]") == 0)
        {
            prog[i] = 'm';
            prog[i+1] = 'r';
            prog[i+2] = prog[i+5];
            prog[i+3] = prog[i+4] = prog[i+5] = prog[i+6] = prog[i+7] = prog[i+8] = prog[i+9] = prog[i+10] = prog[i+11] = '0';
        }
        
        else if (strcmp(temp, "[>+<-]") == 0)
        {
            prog[i] = 'm';
            prog[i+1] = 'r';
            prog[i+2] = prog[i+3];
            prog[i+3] = prog[i+4] = prog[i+5] = prog[i+6] = prog[i+7] = prog[i+8] = prog[i+9] = prog[i+10] = prog[i+11] = '0';
        }
        
        else if (strcmp(temp, "[-<+>]") == 0)
        {
            prog[i] = 'm';
            prog[i+1] = 'l';
            prog[i+2] = prog[i+5];
            prog[i+3] = prog[i+4] = prog[i+5] = prog[i+6] = prog[i+7] = prog[i+8] = prog[i+9] = prog[i+10] = prog[i+11] = '0';
        }
        
        else if (strcmp(temp, "[<+>-]") == 0)
        {
            prog[i] = 'm';
            prog[i+1] = 'l';
            prog[i+2] = prog[i+3];
            prog[i+3] = prog[i+4] = prog[i+5] = prog[i+6] = prog[i+7] = prog[i+8] = prog[i+9] = prog[i+10] = prog[i+11] = '0';
        }
        else
        {
            i -= 10;
        }
        
        i += 10;
        
        free(temp);
    }
}





int main(int argc, char* argv[])
{
  if (argc<2)
    return 0;
  
  unsigned longueur;
  char* program = loadProgram(argv[1], &longueur);
  
  if (program == NULL)
      return EXIT_FAILURE;
  
  //optimize(program, longueur); // crée des optimisations liées aux déplacements de valeurs
  
  //mise en place des liens entre '[' et ']'
  unsigned* crochOuv = malloc(longueur * sizeof(unsigned));
  memset(crochOuv, 0, longueur);
  unsigned* crochFerm = malloc(longueur * sizeof(unsigned));
  memset(crochFerm, 0, longueur);
    
    
  int res = crochets(program, longueur, crochOuv, crochFerm);
  
  if (res)
      return EXIT_FAILURE;

  int numProg = 0; // représente l'index d'instruction actuelle dans le pointeur program
  unsigned longProg = strlen(program); // longueur du programme

  unsigned taille_mem = 50000000; // 50 mo

  char* memoire = malloc(taille_mem * sizeof(char));//toute la mémoire du programme
  memset(memoire, 0, taille_mem);
  unsigned ptr = 0;
  
  Queue* bande_entree = createQueue();
  
  register char car;

  // boucle principale, boucle 'critique'
  while (numProg < longProg)// tant que l'on n'est pas à la fin du programme
  {
    car=program[numProg]; // l'inctruction courante du programme
        
    switch (car)
    {
      case '+' :
        memoire[ptr]+=(unsigned)program[numProg+1];
        break;

      case '-' :
        memoire[ptr]-=(unsigned)program[numProg+1];
        break;

      case '>' :
        ptr+=(int)program[numProg+1];
        break;

      case '<' :
        ptr-=(int)program[numProg+1];
        break;

      case '.' :
        {
            char car;
            for (unsigned i=0;i<program[numProg+1];i++)
            {
                putchar(memoire[ptr]);
            }
        }
        break;

      case ',' :
        virgule(bande_entree, program, numProg, memoire, ptr);
        break;

      case '[' :
        for (register unsigned i=0;i<program[numProg+1];i++)
        {
          if (!memoire[ptr])
            numProg=crochOuv[numProg];
        }
        break;

      case ']':
        for (register unsigned i=0;i<program[numProg+1];i++)
          numProg=crochFerm[numProg];
        numProg-=2;
        break;
      
      
      case 'm': // provient de l'optimisation de déplacement
        {
            if (program[numProg+1] == 'r') // déplacement à droite
            {
                memoire[ptr + program[numProg+2]] = memoire[ptr];
                memoire[ptr] = 0;
            }
            else // déplacement à gauche
            {
                memoire[ptr - program[numProg+2]] = memoire[ptr];
                memoire[ptr] = 0;
            }
            numProg += 10;
        }
        break;
        
        
    }

    //affMem(memoire, numProg, ptr, car);
    
    numProg+=2;
  
  }

  printf("\n");
  free(memoire);
  free(crochOuv);
  free(crochFerm);
  free(program);
  system("pause");
  
  return 0;
}
