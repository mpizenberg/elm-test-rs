const net = require('net')
const client = net.createConnection({ port: 30000 }, onConnection)
client.on('data', onData)
client.on('end', () => { console.log("disconnected from server") })

function onConnection() {
	console.log("connected to server")
	client.write("hello from client")
}

function onData(data) {
	console.log("data received: " + data.toString());
	setTimeout(() => client.end(), 5000)
}
