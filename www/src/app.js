const express = require('express');
const { execute } = require('./execute.js');

const PORT = 3000;
const HOSTNAME = 'localhost';

express()
  .use(express.json())
  .use(express.static('public'))
  .post('/execute', execute)
  .listen(PORT, HOSTNAME);
