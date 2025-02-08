const express = require('express');
const grpc = require('@grpc/grpc-js');
const protoLoader = require('@grpc/proto-loader');
const path = require('path');
const bodyParser = require('body-parser');

const anonvote_server_address = 'localhost:50051';
const app = express();
const port = 3000;

// gRPC client
const packageDefinition = protoLoader.loadSync(path.join(__dirname, 'anonvote.proto'));
const proto = grpc.loadPackageDefinition(packageDefinition).anonvote;
const client = new proto.AnonVote(anonvote_server_address, grpc.credentials.createInsecure());

app.use(bodyParser.json());
app.use(express.static(path.join(__dirname, '..', 'public')));

app.get('/', (req, res) => {
  res.sendFile(path.join(__dirname, '..', 'public', 'index.html'));
});

app.post('/validate_id', (req, res) => {
    const id = req.body.id || '';
    client.ValidateID({ id }, (error, response) => {
      if (error) {
        return res.status(500).send(error);
      }
      res.json(response);
    });
  });

app.post('/register', (req, res) => {
  const message = req.body || undefined;
  client.Register(message, (error, response) => {
    if (error) {
      return res.status(500).send(error);
    }
    res.json(response);
  });
});

app.listen(port, () => {
  console.log(`Server is running on http://localhost:${port}`);
});
