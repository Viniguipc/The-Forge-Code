const express = require('express');
const fs = require('fs');
const path = require('path');

const app = express();
const PORT = 3000;
const DATA_FILE = path.join(__dirname, 'tasks.json');

// Configurações do Express
app.set('view engine', 'ejs');
// Middleware para capturar dados de formulários POST (application/x-www-form-urlencoded)
app.use(express.urlencoded({ extended: true }));

// Funções utilitárias para manipular o JSON
const readTasks = () => {
    if (!fs.existsSync(DATA_FILE)) {
        fs.writeFileSync(DATA_FILE, JSON.stringify([]));
        return [];
    }
    const data = fs.readFileSync(DATA_FILE, 'utf8');
    return JSON.parse(data || '[]');
};

const writeTasks = (tasks) => {
    fs.writeFileSync(DATA_FILE, JSON.stringify(tasks, null, 2));
};

// ==========================================
// ROTAS (CRUD)
// ==========================================

// READ: Renderizar a página principal com as tarefas
app.get('/', (req, res) => {
    const tasks = readTasks();
    res.render('index', { tasks });
});

// CREATE: Adicionar uma nova tarefa
app.post('/add', (req, res) => {
    const tasks = readTasks();
    const newTask = {
        id: Date.now().toString(), // Gera um ID simples usando o timestamp
        text: req.body.taskText,
        completed: false
    };
    
    tasks.push(newTask);
    writeTasks(tasks);
    res.redirect('/');
});

// UPDATE: Marcar tarefa como concluída (ou desfazer)
app.post('/complete/:id', (req, res) => {
    const tasks = readTasks();
    const taskIndex = tasks.findIndex(t => t.id === req.params.id);
    
    if (taskIndex > -1) {
        // Inverte o status atual (de false para true, e vice-versa)
        tasks[taskIndex].completed = !tasks[taskIndex].completed;
        writeTasks(tasks);
    }
    res.redirect('/');
});

// DELETE: Excluir uma tarefa
app.post('/delete/:id', (req, res) => {
    let tasks = readTasks();
    tasks = tasks.filter(t => t.id !== req.params.id);
    writeTasks(tasks);
    res.redirect('/');
});

// Iniciar servidor
app.listen(PORT, () => {
    console.log(`Servidor rodando em: http://localhost:${PORT}`);
});
