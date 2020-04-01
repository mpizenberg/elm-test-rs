const net = require('net')
const server = net.createServer(onConnection)
server.on('error', (err) => { throw err })
server.listen(30000, () => { console.log("server bound to", server.address()) })

function onConnection(connection) {
	console.log("client connected:", connection.address())
	console.log("localAddress:", connection.localAddress)
	connection.on('data', onData(connection))
	connection.on('end', () => { console.log("client disconnected") })
	connection.write('hello from server')
}

function onData(connection) { return (data) => {
	console.log("data received: " + data.toString())
}}
