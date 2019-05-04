const express = require('express');
const app = express();

process.env['DEBUG'] = 'express:*';
express.static.mime.types['wasm'] = 'application/wasm';
app.use(express.static(__dirname + '/'));

const port = 8000;
app.listen(port, () => { console.log('listening to port %s', port); });