const express = require('express');
const app = express();

var args = process.argv.slice(2);

const id   = args[0];
const port = args[1];

app.get('/', (request, response) => {
  response.set('Backend-Id', id);
  response.send('Hello from Express!');
})

app.listen(port, (err) => {
  if (err) {
    return console.log('something bad happened', err);
  }

  console.log(`server is listening on ${port}`);
})
