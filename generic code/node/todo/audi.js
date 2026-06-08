const http = require ('http');
const obterTask = require('./task');

const servidor = http.createServer((req, res) => {
	res.writeHead(200, {'Content-Type': 'text/plain; charset=utf-8'});
	
	res.end('Hello, World!');
});

servidor.listen(3000, () => {
	console.log('localhost 3000');
});


